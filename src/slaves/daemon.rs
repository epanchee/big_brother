use std::{sync::Arc, thread, time::Duration};

use tokio::sync::Mutex;

use super::{config_parser::parse_config_dir, fetchers::BaseFetcher};

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

    async fn fetch_data(configs: Arc<Mutex<Vec<BaseFetcher>>>) {
        for config in configs.try_lock().unwrap().iter() {
            let fetched = config
                .fetch()
                .await
                .map(|list| list.iter().cloned().flatten().collect::<Vec<_>>());
            if let Ok(fetched) = fetched {
                todo!()
            } else {
                println!("Couldn't fetch any data in {:?}", config)
            }
        }
    }

    pub fn start(self) {
        let configs = parse_config_dir(&self.conf_path[..]);
        let configs = Arc::new(Mutex::new(configs));
        loop {
            let configs = configs.clone();
            tokio::spawn(async move { Self::fetch_data(configs).await });
            print!("Going to sleep for {} secs...", self.interval.as_secs());
            thread::sleep(self.interval)
        }
    }
}
