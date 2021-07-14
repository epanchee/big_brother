use std::{
    fs::{self},
    path::Path,
};

use anyhow::{anyhow, Context, Result};

use super::fetchers::{Fetchable, FetcherConfig, SimpleFetcher};

pub fn parse_yaml(config_file: &str) -> Result<Box<dyn Fetchable>> {
    let content = fs::read_to_string(config_file).unwrap();
    let config: FetcherConfig = serde_yaml::from_str(&content[..])?;
    Ok(Box::new(SimpleFetcher { config }))
}

pub fn parse_config_dir(dir_str: &str) -> Vec<Box<dyn Fetchable>> {
    let dir = Path::new(dir_str);
    let mut fetchers: Vec<Box<dyn Fetchable>> = vec![];
    let files = fs::read_dir(dir).unwrap();
    for dir_entry in files {
        let result = dir_entry.map_err(From::from).and_then(|dir_entry| {
            let path = dir_entry.path();
            let parse_file = || -> Result<Box<dyn Fetchable>> {
                let ext = path
                    .extension()
                    .ok_or_else(|| anyhow!("Path has no extension"))?;
                if ext == "yaml" {
                    parse_yaml(
                        path.to_str()
                            .ok_or_else(|| anyhow!("Path to str conversion error"))?,
                    )
                } else {
                    Err(anyhow!("I can only parse .yaml files"))
                }
            };
            parse_file().with_context(|| format!("Error occured with {:?}", path))
        });

        if let Ok(config) = result {
            fetchers.push(config);
        } else {
            println!("{:?}", result);
        }
    }
    fetchers
}

#[cfg(test)]
pub mod tests {
    use crate::slaves::{
        config_parser::{parse_config_dir, parse_yaml},
        fetchers::{FetchItem, FetchItemType, FetcherConfig, SimpleFetcher},
    };

    fn gen_config1() -> SimpleFetcher {
        let item1 = FetchItem {
            name: "item1".to_string(),
            path: "body > div > p:nth-child(3) > a".to_string(),
            primary: false,
            item_type: FetchItemType::Text,
            related: vec![],
        };

        let item2 = FetchItem {
            name: "item2".to_string(),
            ..item1.clone()
        };

        let item3 = FetchItem {
            name: "item3".to_string(),
            primary: true,
            related: vec![item1.clone(), item2.clone()],
            ..item1.clone()
        };

        let config = FetcherConfig {
            items: vec![item1, item2, item3],
            url: "http://example.com".to_string(),
        };

        SimpleFetcher { config }
    }

    fn gen_config2() -> FetcherConfig {
        let item_x = FetchItem {
            name: "entity_x".to_string(),
            path: "body > div > p:nth-child(3) > a".to_string(),
            primary: false,
            item_type: FetchItemType::Text,
            related: vec![],
        };
        let item_y = FetchItem {
            name: "entity_y".to_string(),
            related: vec![item_x.clone()],
            ..item_x.clone()
        };
        let item_z = FetchItem {
            name: "entity_z".to_string(),
            related: vec![item_y.clone()],
            primary: true,
            ..item_x.clone()
        };

        FetcherConfig {
            items: vec![item_x, item_y, item_z],
            url: "http://another-example.com".to_string(),
        }
    }

    #[test]
    fn test_parse_yaml() {
        let config = gen_config1();
        let mut fetch_items = parse_yaml("configs/example.yaml").unwrap();
        assert_eq!(config, fetch_items);
    }

    #[test]
    fn test_parse_config_dir() {
        let config1 = gen_config1();
        let config2 = gen_config2();

        let mut configs = parse_config_dir("test/configs");

        assert_eq!(vec![config2, config1], configs);

        // detailed test
        // configs[1].iter().zip(&config1).for_each(|(i1, i2)| assert_eq!(i1, i2));
        // configs[0].iter().zip(&config2).for_each(|(i1, i2)| assert_eq!(i1, i2));
    }
}
