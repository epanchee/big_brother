use std::{sync::Arc, thread, time::Duration};

use tokio::sync::Mutex;

use super::{
    config_parser::parse_config_dir,
    fetchers::{BaseFetcher, FoundItem},
};

pub struct FetchDaemon {
    interval: Duration,
    conf_path: String,
}

impl FetchDaemon {
    pub fn new(interval: Duration, conf_path: String) -> Self {
        FetchDaemon {
            interval,
            conf_path,
        }
    }

    pub fn new_with_default_conf(interval: Duration) -> Self {
        FetchDaemon {
            interval,
            conf_path: "configs".to_string(),
        }
    }

    async fn fetch_data(configs: Arc<Mutex<Vec<BaseFetcher>>>) -> Vec<Vec<FoundItem<String>>> {
        let mut fetched_confs = vec![];
        for config in configs.try_lock().unwrap().iter() {
            let fetched = config
                .fetch()
                .await
                .map(|list| list.iter().cloned().flatten().collect::<Vec<_>>());
            if let Ok(fetched) = fetched {
                fetched_confs.push(fetched)
            } else {
                println!("Couldn't fetch any data in {}", config)
            }
        }
        fetched_confs
    }

    #[tokio::main]
    pub async fn start(self) {
        let configs = parse_config_dir(&self.conf_path[..]);
        let configs = Arc::new(Mutex::new(configs));
        loop {
            let configs = configs.clone();

            tokio::spawn(async move {
                let fetched = Self::fetch_data(configs).await;
                println!("{:?}", fetched);
            })
            .await
            .expect("Spawned async context failed");

            println!("Going to sleep for {} secs...", self.interval.as_secs());
            thread::sleep(self.interval)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::Mutex;

    use crate::slaves::{
        config_parser::parse_config_dir,
        fetchers::{BaseFetcher, FetchItem, FoundItem},
    };

    use super::FetchDaemon;

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

    #[tokio::test]
    async fn test_fetch_data() {
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

        let config1 = BaseFetcher {
            items: vec![item1.clone(), item2.clone(), item3.clone()],
            url: "http://example.com".to_string(),
        };
        let config2 = gen_config2();

        let configs = parse_config_dir("configs");
        let configs = Arc::new(Mutex::new(configs));
        let mut fetched = FetchDaemon::fetch_data(configs).await;
        fetched.sort();

        let mut correct = vec![
            FoundItem {
                fetch_item: item1,
                content: "More information...".to_string(),
            },
            FoundItem {
                fetch_item: item2,
                content: "More information...".to_string(),
            },
            FoundItem {
                fetch_item: item3,
                content: "More information...".to_string(),
            },
        ];
        correct.sort();

        // fetched[0].iter().zip(&correct).for_each(|(i1, i2)| assert_eq!(i1, i2));
        println!("{:#?}", fetched);
        println!("{:#?}", correct);
        assert_eq!(fetched, vec![correct])
    }
}
