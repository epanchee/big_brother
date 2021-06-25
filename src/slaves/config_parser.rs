use std::fs;

use crate::slaves::fetchers::FetchItem;

pub fn parse_yaml(config_file: &str) -> Vec<FetchItem> {
    let content = fs::read_to_string(config_file).unwrap();
    serde_yaml::from_str(&content[..]).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::slaves::{config_parser::parse_yaml, fetchers::FetchItem};

    #[test]
    fn test_parse_yaml() {
        let item1 = FetchItem {
            name: "item1".to_string(),
            path: "body > div > p:nth-child(3) > a".to_string(),
            primary: false,
            item_type: "".to_string(),
            related: vec![],
        };

        let item2 = FetchItem {
            name: "item2".to_string(),
            ..item1.clone()
        };

        let item3 = FetchItem {
            name: "item3".to_string(),
            related: vec![Box::new(item1.clone()), Box::new(item2.clone())],
            ..item1.clone()
        };

        let mut fetch_items = parse_yaml("configs/example.yaml");
        fetch_items.sort();
        vec![item1, item2, item3].iter().zip(fetch_items).for_each(
            move |(ok, test)| assert_eq!(*ok, test)
        );
    }
}
