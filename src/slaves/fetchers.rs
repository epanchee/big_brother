use anyhow::{anyhow, Result};
use scraper::{ElementRef, Html, Selector};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct FetchItem {
    pub name: String,
    pub path: String,
    pub primary: bool,
    pub item_type: String,
    pub related: Vec<Box<FetchItem>>,
}

impl FetchItem {
    fn seek(&self, data: ElementRef) -> String {
        data.inner_html()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &FetchItem> + '_> {
        Box::new(std::iter::once(self).chain(self.related.iter().flat_map(|x| x.iter())))
    }
}

pub struct BaseFetcher {
    pub items: Vec<FetchItem>,
    pub url: String,
    pub tree: Html,
    pub fetched: Vec<Option<String>>,
}

impl BaseFetcher {
    async fn get_from_remote(&self) -> Result<Html, Box<dyn std::error::Error>> {
        let resp_text = reqwest::get(&self.url).await?.text().await?;
        Ok(Html::parse_document(&resp_text[..]))
    }

    fn select(&self, selector: &String) -> Result<ElementRef> {
        let selector = Selector::parse(&selector[..])
            .map_err(|x| anyhow!("Selector parsing errored {:?}", x))?;
        Ok(self.tree.select(&selector).next().unwrap())
    }

    pub async fn fetch(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.tree = self.get_from_remote().await?;
        for root_item in self.items.iter() {
            for item in root_item.iter() {
                self.fetched.push({
                    if let Ok(data) = self.select(&item.path) {
                        Some(item.seek(data))
                    } else {
                        None
                    }
                })
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use scraper::{Html, Selector};

    use crate::slaves::fetchers::{BaseFetcher, FetchItem};

    #[tokio::test]
    async fn reqwest_works() {
        let resp = reqwest::get("https://httpbin.org/ip")
            .await
            .unwrap()
            .json::<HashMap<String, String>>()
            .await
            .unwrap();
        assert!(resp.contains_key("origin"))
    }

    #[tokio::test]
    async fn scraper_works() {
        let resp_text = reqwest::get("http://example.com/")
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let tree = Html::parse_document(&resp_text[..]);
        let selector = Selector::parse("body > div > p:nth-child(3) > a").unwrap();
        let selected_text = tree.select(&selector).take(1).collect::<Vec<_>>()[0].inner_html();
        assert_eq!(selected_text, "More information...");
    }

    #[tokio::test]
    async fn test_base_fetcher() {
        let item1 = FetchItem {
            name: "item1".to_string(),
            path: "body > div > p:nth-child(3) > a".to_string(),
            primary: false,
            item_type: "".to_string(),
            related: vec![],
        };

        let mut fetcher = BaseFetcher {
            items: vec![item1],
            url: "http://example.com/".to_string(),
            tree: Html::new_document(),
            fetched: vec![],
        };

        fetcher.fetch().await.expect("Fetch failed");

        assert_eq!(fetcher.fetched[0].as_ref().unwrap(), "More information...");
    }
}
