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
        Plain => serialize_plain(item),
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
