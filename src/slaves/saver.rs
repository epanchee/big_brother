use crate::slaves::serializer::serialize_all;

use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

use super::{
    fetchers::FoundItem,
    serializer::SerType::{self, *},
};
use anyhow::{anyhow, Result};
use async_recursion::async_recursion;

pub enum SaverType {
    Stdout,
    File(String),
    Multiple(Vec<Saver>),
    Telegram,
    Postgres,
}

use SaverType::*;

pub struct Saver {
    stype: SaverType,
    sertype: SerType,
}

impl Saver {
    fn new(stype: SaverType, sertype: SerType) -> Self {
        Saver { stype, sertype }
    }

    fn new_default() -> Self {
        Self::new(Stdout, Json)
    }

    #[async_recursion]
    async fn push(&'static self, data: &'static Vec<Vec<FoundItem>>) -> Result<()> {
        let ser_data = serialize_all(data.clone(), self.sertype);
        match &self.stype {
            Stdout => println!("{}", ser_data),
            File(path) => {
                let mut op = OpenOptions::new();
                let mut file = op.create(true).append(true).open(path).await?;
                file.write_all(ser_data.as_bytes()).await?;
                file.sync_all().await?;
            }
            Multiple(sinks) => {
                let handlers = sinks
                    .iter()
                    .map(|sink| tokio::spawn(async move { sink.push(&data).await }));
                for handler in handlers {
                    handler.await?;
                }
            }
            Telegram => unimplemented!(),
            Postgres => unimplemented!(),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::slaves::{
        fetchers::{FetchItem, FetchItemType::*, FoundItem, FoundItemContent::*},
        serializer::SerType,
    };

    use super::{Saver, SaverType::*};

    use tokio::fs::File;
    use tokio::io::AsyncReadExt;

    fn create_test_data() -> Vec<Vec<FoundItem>> {
        let translations = FetchItem {
            name: "translations".to_string(),
            path: "#Content > div:nth-child(5)".to_string(),
            primary: false,
            item_type: Class,
            related: vec![],
        };

        let item1 = FetchItem {
            name: "item1".to_string(),
            path: "#Content > div:nth-child(5) > strong".to_string(),
            primary: true,
            item_type: Text,
            related: vec![translations.clone()],
        };

        let correct = vec![FoundItem {
            fetch_item: item1,
            content: Str("Translations:".to_string()),
            related: vec![Some(FoundItem {
                fetch_item: translations,
                content: Arr(vec!["boxed".to_string()]),
                related: vec![],
            })],
        }];
        let mut correct = vec![correct];
        correct.sort();
        correct
    }

    #[tokio::test]
    async fn test_save_to_file() {
        let path = "test/test.out".to_string();
        let saver = Saver::new(File(path.clone()), SerType::Json);
        let test_data = create_test_data();

        saver.push(&test_data).await.unwrap();
        let mut content = vec![];
        File::open(path.clone())
            .await
            .unwrap()
            .read_to_end(&mut content)
            .await
            .unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(
            String::from_utf8(content).unwrap(),
            r#"[[{"name":"item1","content":"Translations:","related":[{"name":"translations","content":["boxed"],"related":[]}]}]]"#.to_string()
        );
    }
}
