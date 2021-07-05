use crate::slaves::fetchers::FoundItemContent;

use super::fetchers::{FoundItem, FoundItemContent::*};

#[derive(Copy, Clone)]
pub enum SerType {
    Plain,
    Json,
}

use SerType::*;

pub fn serialize_all(fetched_configs: Vec<Vec<FoundItem>>, sertype: SerType) -> String {
    match sertype {
        Plain => {
            let mut result = vec![];
            for config in fetched_configs {
                for item in config {
                    result.push(serialize_plain(item))
                }
            }
            result.join(" ")
        }
        Json => serde_json::to_string(&fetched_configs).unwrap(),
    }
}

fn serialize_plain(item: FoundItem) -> String {
    let convert2str = |name: String, val: FoundItemContent| match val {
        Str(val) => format!("{}={}", name, val),
        Arr(val) => format!("{}={}", name, val.into_iter().collect::<Vec<_>>().join("")),
    };

    if item.related.is_empty() {
        convert2str(item.fetch_item.name, item.content)
    } else {
        format!(
            "{}: {}",
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

    #[test]
    fn test_serialize_plain() {
        let data = create_test_data();

        assert_eq!(
            serialize_all(data, Plain),
            "item1=Translations:: translations=boxed".to_string()
        )
    }

    #[test]
    fn test_serialize_json() {
        let data = create_test_data();

        assert_eq!(
            serialize_all(data, Json),
            r#"[[{"name":"item1","content":"Translations:","related":[{"name":"translations","content":["boxed"],"related":[]}]}]]"#
        )
    }
}
