use crate::slaves::fetchers::FoundItemContent;

use super::fetchers::{FoundItem, FoundItemContent::*};

#[derive(Copy, Clone)]
pub enum SerType {
    Plain,
    Json,
    Yaml,
}

use SerType::*;

pub fn serialize_all(fetched_configs: Vec<Vec<FoundItem>>, sertype: SerType) -> String {
    let mut result = vec![];
    for config in fetched_configs {
        for item in config {
            result.push(serialize(item, sertype))
        }
    }
    result.join("")
}

fn serialize(item: FoundItem, sertype: SerType) -> String {
    match sertype {
        Plain => serialize_plain(item).trim().to_string(),
        Json => unimplemented!(),
        Yaml => unimplemented!(),
    }
}

fn serialize_plain(item: FoundItem) -> String {
    let convert2str = |name: String, val: FoundItemContent| match val {
        Str(val) => format!("{}={} ", name, val),
        Arr(val) => format!("{}={} ", name, val.into_iter().collect::<Vec<_>>().join("")),
    };

    if item.related.is_empty() {
        convert2str(item.fetch_item.name, item.content)
    } else {
        format!(
            "{}: {} ",
            convert2str(item.fetch_item.name, item.content),
            item.related
                .into_iter()
                .flatten()
                .map(serialize_plain)
                .collect::<Vec<_>>()
                .join("")
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::slaves::fetchers::{FetchItem, FetchItemType::*, FoundItem, FoundItemContent::*};
    use crate::slaves::serializer::{serialize_all, SerType::*};

    #[test]
    fn test_serialize_plain() {
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

        assert_eq!(
            serialize_all(correct, Plain),
            "item1=Translations: : translations=boxed".to_string()
        )
    }
}
