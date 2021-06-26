use std::{
    fs::{self},
    path::Path,
};

use anyhow::{anyhow, Result};

use crate::slaves::fetchers::FetchItem;

type Config = Vec<FetchItem>;

pub fn parse_yaml(config_file: &str) -> Config {
    let content = fs::read_to_string(config_file).unwrap();
    serde_yaml::from_str(&content[..]).unwrap()
}

pub fn parse_config_dir(dir_str: &str) -> Vec<Config> {
    let dir = Path::new(dir_str);
    let mut configs: Vec<Config> = vec![];
    for entry in fs::read_dir(dir).unwrap() {
        let result = || -> Result<Config> {
            let path = entry?.path();
            let ext = path.extension().ok_or(anyhow!("Path has no extension"))?;
            if ext == "yaml" {
                Ok(parse_yaml(
                    path.to_str()
                        .ok_or(anyhow!("Path to str conversion error"))?,
                ))
            } else {
                Err(anyhow!("I can only parse .yaml files"))
            }
        }();

        if let Ok(config) = result {
            configs.push(config);
        } else {
            println!(
                "Error happened with entry {:#?}\nDetails: {:?}",
                entry, result
            )
        }
    }
    configs
}

#[cfg(test)]
mod tests {
    use crate::slaves::{
        config_parser::{parse_config_dir, parse_yaml},
        fetchers::FetchItem,
    };

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
        assert_eq!(vec![item1, item2, item3], fetch_items);
    }

    #[test]
    fn test_parse_config_dir() {
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
        let item_x = FetchItem {
            name: "entity_x".to_string(),
            path: "body > div > p:nth-child(3) > a".to_string(),
            primary: false,
            item_type: "".to_string(),
            related: vec![],
        };
        let item_y = FetchItem {
            name: "entity_y".to_string(),
            related: vec![Box::new(item_x.clone())],
            ..item1.clone()
        };
        let item_z = FetchItem {
            name: "entity_z".to_string(),
            related: vec![Box::new(item_y.clone())],
            primary: true,
            ..item1.clone()
        };

        let config1 = vec![item1, item2, item3];
        let config2 = vec![item_x, item_y, item_z];

        let mut configs = parse_config_dir("configs");
        configs.sort();

        assert_eq!(vec![config2, config1], configs);

        // detailed test
        // configs[1].iter().zip(&config1).for_each(|(i1, i2)| assert_eq!(i1, i2));
        // configs[0].iter().zip(&config2).for_each(|(i1, i2)| assert_eq!(i1, i2));
    }
}
