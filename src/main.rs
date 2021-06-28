mod slaves;

use std::time::Duration;

use slaves::daemon::FetchDaemon;

#[tokio::main]
async fn main() {
    // fetchers::main();
    // println!("{:#?}", config_parser::parse_yaml("configs/example.yaml"));
    // println!("{:#?}", config_parser::parse_config_dir("configs"));

    FetchDaemon::new_with_default_conf(
        Duration::from_secs(10)
    ).start();
}
