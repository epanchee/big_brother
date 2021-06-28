use std::{
    fs::{self},
    path::Path,
};

use anyhow::{anyhow, Context, Result};

use super::fetchers::BaseFetcher;

pub fn parse_yaml(config_file: &str) -> Result<BaseFetcher> {
    let content = fs::read_to_string(config_file).unwrap();
    serde_yaml::from_str(&content[..]).map_err(|_| anyhow!("Failed to parse yaml content"))
}

pub fn parse_config_dir(dir_str: &str) -> Vec<BaseFetcher> {
    let dir = Path::new(dir_str);
    let mut configs: Vec<BaseFetcher> = vec![];
    let files = fs::read_dir(dir).unwrap();
    for dir_entry in files {
        let result = dir_entry.map_err(From::from).and_then(|dir_entry| {
            let path = dir_entry.path();
            let parse_file = || -> Result<BaseFetcher> {
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
            configs.push(config);
        } else {
            println!("{:?}", result);
        }
    }
    configs
}

#[cfg(test)]
pub mod tests {
    use crate::slaves::{
        config_parser::{parse_config_dir, parse_yaml},
        fetchers::{BaseFetcher, FetchItem},
    };

    fn gen_config1() -> BaseFetcher {
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
            related: vec![item1.clone(), item2.clone()],
            ..item1.clone()
        };

        BaseFetcher {
            items: vec![item1, item2, item3],
            url: "http://example.com".to_string(),
        }
    }

    fn gen_config2() -> BaseFetcher {
        let item_x = FetchItem {
            name: "entity_x".to_string(),
            path: "body > div > p:nth-child(3) > a".to_string(),
            primary: false,
            item_type: "".to_string(),
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

        BaseFetcher {
            items: vec![item_x, item_y, item_z],
            url: "http://another-example.com".to_string(),
        }
    }

    #[test]
    fn test_parse_yaml() {
        let config = gen_config1();
        let mut fetch_items = parse_yaml("configs/example.yaml").unwrap();
        fetch_items.items.sort();
        assert_eq!(config, fetch_items);
    }

    #[test]
    fn test_parse_config_dir() {
        let config1 = gen_config1();
        let config2 = gen_config2();

        let mut configs = parse_config_dir("configs");
        configs.sort();

        assert_eq!(vec![config2, config1], configs);

        // detailed test
        // configs[1].iter().zip(&config1).for_each(|(i1, i2)| assert_eq!(i1, i2));
        // configs[0].iter().zip(&config2).for_each(|(i1, i2)| assert_eq!(i1, i2));
    }
}
