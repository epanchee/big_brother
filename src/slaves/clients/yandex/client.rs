use std::{io, sync::Arc, time::Duration};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::{header::*, Client, Response, Url};
use scraper::{Html, Selector};

use crate::slaves::{
    clients::custom_cookies::MyJar,
    fetchers::{FetchItem, FetchResults, Fetchable, FetcherConfig, FoundItem},
};

const SELECTOR_ERROR: &str = "Hardcoded selector parse error";

#[derive(Debug)]
pub struct YandexClient {
    origin: String,
    cookies_jar: Arc<MyJar>,
    pub client: Client,
    pub config: FetcherConfig,
}

impl YandexClient {
    fn gen_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/85.0.4183.102 YaBrowser/20.9.3.189 (beta) Yowser/2.5 Safari/537.36".parse().unwrap());
        headers.insert(
            ACCEPT,
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"
                .parse()
                .unwrap(),
        );
        headers.insert(
            ACCEPT_LANGUAGE,
            "ru,en;q=0.9,zh;q=0.8,nl;q=0.7,es;q=0.6".parse().unwrap(),
        );
        headers.insert(ACCEPT_ENCODING, "identity".parse().unwrap());
        headers.insert(CONNECTION, "keep-alive".parse().unwrap());
        headers
    }

    pub fn new(config: FetcherConfig) -> Self {
        let url: Url = config.url.parse().unwrap();
        let cookies_jar = Arc::new(MyJar::new(url.host().unwrap().to_string()));
        YandexClient {
            origin: Self::get_origin(url),
            cookies_jar: cookies_jar.clone(),
            client: Self::build_client(cookies_jar),
            config,
        }
    }

    fn get_origin(url: Url) -> String {
        url.origin().unicode_serialization()
    }

    fn build_client(cookies_jar: Arc<MyJar>) -> Client {
        Client::builder()
            .cookie_provider(cookies_jar)
            .default_headers(Self::gen_headers())
            // .proxy(reqwest::Proxy::all("localhost:8888").unwrap()).danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap()
    }

    async fn get_captcha_image(&self, resp: Response) -> Result<String> {
        let tree = Html::parse_document(&resp.text().await?);

        let selector =
            Selector::parse(".AdvancedCaptcha-Image").map_err(|_| anyhow!(SELECTOR_ERROR))?;
        let selected = tree.select(&selector).take(1).collect::<Vec<_>>()[0];
        let img_path = selected
            .value()
            .attr("src")
            .ok_or_else(|| anyhow!("Img path select error"))?;

        let selector =
            Selector::parse(".AdvancedCaptcha-Form").map_err(|_| anyhow!(SELECTOR_ERROR))?;
        let selected = tree.select(&selector).take(1).collect::<Vec<_>>()[0];
        let action = selected
            .value()
            .attr("action")
            .ok_or_else(|| anyhow!("Action select error"))?;

        let action_path = self.origin.to_owned() + action;
        println!("img: {}\n2nd action: {}", img_path, action_path);
        Ok(action_path)
    }

    async fn crack_captcha(&self, action: &str) -> Result<String> {
        let action_path = self.origin.to_owned() + action;
        println!("1st action: {}", action_path);
        let resp = self.client.get(action_path).send().await?;

        let action_path = self.get_captcha_image(resp).await?;
        println!("Enter captcha:");
        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        let params = [("rep", guess)];
        let resp = self.client.post(action_path).form(&params).send().await?;
        Ok(resp.text().await?)
    }

    pub async fn retrieve(&self) -> Result<Html> {
        let resp = self.client.get(self.config.url.clone()).send().await?;
        let text = resp.text().await?;
        let captcha_form_selector =
            Selector::parse(".CheckboxCaptcha-Form").map_err(|_| anyhow!(SELECTOR_ERROR))?;
        let action = Html::parse_document(&text)
            .select(&captcha_form_selector)
            .next()
            .map(|x| x.value().attr("action").unwrap().to_owned());
        let result = if let Some(action) = action {
            let text = self.crack_captcha(&action).await?;
            self.cookies_jar.store_cookies()?;
            text
        } else {
            text
        };
        Ok(Html::parse_document(&result))
    }

    fn process_single_item(item: &FetchItem, tree: &Html) -> Option<FoundItem> {
        if let Ok(data) = item.select(&tree) {
            Some(FoundItem {
                fetch_item: item.clone(),
                content: item.seek(data),
                related: vec![],
            })
        } else {
            None
        }
    }
}

#[async_trait]
impl Fetchable for YandexClient {
    async fn fetch(&self) -> Result<FetchResults> {
        let tree = self.retrieve().await?;
        let mut fetched = vec![];
        let primary_items: Vec<_> = self
            .config
            .items
            .iter()
            .filter(|&item| item.primary)
            .collect();
        for primary_item in primary_items {
            let result =
                if let Some(mut found_item) = Self::process_single_item(primary_item, &tree) {
                    let mut related_items = vec![];
                    for item in primary_item.related.iter() {
                        related_items.push(Self::process_single_item(item, &tree))
                    }
                    found_item.related = related_items;
                    Some(found_item)
                } else {
                    None
                };
            fetched.push(result)
        }
        Ok(fetched)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
