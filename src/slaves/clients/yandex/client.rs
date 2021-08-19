use std::{io, sync::Arc, time::Duration};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::{header::*, Client, Url};
use scraper::{Html, Selector};

use crate::slaves::{
    clients::custom_cookies::MyJar,
    fetchers::{FetchResults, Fetchable},
};

const SELECTOR_ERROR: &str = "Hardcoded selector parse error";

#[derive(Debug)]
pub struct YandexClient {
    url: Url,
    origin: String,
    cookies_jar: Arc<MyJar>,
    pub client: Client,
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

    pub fn new(url: &str) -> Self {
        let url: Url = url.parse().unwrap();
        let cookies_jar = Arc::new(MyJar::new(url.host().unwrap().to_string()));
        YandexClient {
            url: url.clone(),
            origin: Self::get_origin(url),
            cookies_jar: cookies_jar.clone(),
            client: Self::build_client(cookies_jar),
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

    async fn crack_captcha(&self, action: &str) -> Result<String> {
        let mut action_path = self.origin.to_owned() + action;
        println!("1st action: {}", action_path);

        let resp = self.client.get(action_path).send().await?;

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

        action_path = self.origin.to_owned() + action;
        println!("img: {}\n2nd action: {}", img_path, action_path);

        println!("Enter captcha:");
        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        let params = [("rep", guess)];
        let resp = self.client.post(action_path).form(&params).send().await?;
        Ok(resp.text().await?)
    }

    pub async fn fetch(&self) -> Result<String> {
        let resp = self.client.get(self.url.clone()).send().await?;
        let mut text = resp.text().await?;
        let tree = Html::parse_document(&text);
        let captcha_form_selector =
            Selector::parse(".CheckboxCaptcha-Form").map_err(|_| anyhow!(SELECTOR_ERROR))?;
        text = if let Some(selected) = tree.select(&captcha_form_selector).next() {
            let action = selected.value().attr("action").unwrap();
            let text = self.crack_captcha(action).await?;
            self.cookies_jar.store_cookies()?;
            text
        } else {
            text
        };
        Ok(text)
    }
}

#[async_trait]
impl Fetchable for YandexClient {
    async fn fetch(&self) -> Result<FetchResults> {
        todo!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub fn get_price(text: String, css_selector: &str) -> Result<String> {
    let tree = Html::parse_document(&text);
    let price_selector = Selector::parse(css_selector)
        .map_err(|_| anyhow!(SELECTOR_ERROR))?;
    Ok(tree
        .select(&price_selector)
        .next()
        .ok_or_else(|| anyhow!("Price select error"))?
        .inner_html())
}
