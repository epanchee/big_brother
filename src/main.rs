mod slaves;

use std::time::Duration;

use slaves::daemon::FetchDaemon;

#[tokio::main]
async fn main() {
    FetchDaemon::new_with_default_conf(
        Duration::from_secs(10)
    ).start().await;
}
