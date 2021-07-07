mod slaves;

use std::time::Duration;

use slaves::{daemon::FetchDaemon, saver::Saver, saver::SaverType};

#[tokio::main]
async fn main() {
    let savers = vec![
        Saver::new_default(),
        Saver::new_file_json("/tmp/fetched.txt".to_string()),
    ];
    let savetype = SaverType::Multiple(savers);
    let saver = Saver::new_saver_json(savetype);
    
    FetchDaemon::new_default(Duration::from_secs(10), saver)
        .start()
        .await;
}
