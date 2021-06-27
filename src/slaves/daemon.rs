use std::time::Duration;

use super::config_parser::parse_config_dir;

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

    pub fn start(self) {
        let configs = parse_config_dir(&self.conf_path[..]);

        loop {
        }
    }
}
