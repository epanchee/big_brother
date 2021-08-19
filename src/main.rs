mod slaves;

use std::time::Duration;

use slaves::{
    clients::yandex::client::YandexClient, daemon::FetchDaemon, saver::Saver, saver::SaverType,
};

use crate::slaves::clients::yandex::client::get_price;

#[tokio::main]
async fn main() {
    let client = YandexClient::new("https://market.yandex.ru/product--besprovodnye-naushniki-apple-airpods-pro/612787165?text=airpods%20pro&cpa=1&cpc=bF79-BYwlc-v4t-p3FhCE64O6QoblT2bXUfyTM8fSafHqE7JolwvCQTO_W14eME2ZwtuB9KuigKTEHLAlp7IkGKZC_87I5Cdmv_vx-9fUuvkbUmTYGUyFEf4DfvuJgMOJqtn5SObc9wX7YjN5dI5m_nQb5PGAQpX7pbNGhHPpg3kqcZGeriI0mn8ptfbrGth&sku=100812315808&do-waremd5=URCuPaGlZooU6Bzp9p6-fg&nid=18041766");
    loop {
        let result = client.fetch().await.unwrap();
        println!("{}", get_price(result, "div._3NaXx:nth-child(2) > span:nth-child(1) > span:nth-child(1)").unwrap());
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

#[tokio::main]
async fn main1() {
    let savers = vec![
        Saver::new_default(),
        Saver::new_file_json("/tmp/fetched.txt".to_string()),
    ];
    let saver = Saver::new_saver_json(SaverType::Multiple(savers));

    FetchDaemon::new_default(Duration::from_secs(10), saver)
        .start()
        .await;
}
