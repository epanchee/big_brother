use anyhow::{anyhow, Result};
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

pub type Config = Vec<FetchItem>;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct FetchItem {
    pub name: String,
    pub path: String,
    pub primary: bool,
    pub item_type: String,
    pub related: Vec<FetchItem>,
}

impl FetchItem {
    fn seek(&self, data: ElementRef) -> String {
        data.inner_html()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &FetchItem> + '_> {
        Box::new(std::iter::once(self).chain(self.related.iter().flat_map(|x| x.iter())))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct HtmlTree {
    pub inner_tree: Html,
}

impl Default for HtmlTree {
    fn default() -> Self {
        HtmlTree {
            inner_tree: Html::new_document(),
        }
    }
}

impl PartialOrd for HtmlTree {
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ordering::Equal)
    }
}

impl Ord for HtmlTree {
    fn cmp(&self, _: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct BaseFetcher {
    pub items: Config,
    pub url: String,
    #[serde(skip)]
    pub tree: HtmlTree,
    #[serde(skip)]
    pub fetched: Vec<Option<String>>,
}

impl BaseFetcher {
    async fn get_from_remote(&self) -> Result<Html, Box<dyn std::error::Error>> {
        let resp_text = reqwest::get(&self.url).await?.text().await?;
        Ok(Html::parse_document(&resp_text[..]))
    }

    fn select(&self, selector: &str) -> Result<ElementRef> {
        let selector =
            Selector::parse(selector).map_err(|x| anyhow!("Selector parsing errored {:?}", x))?;
        self.tree
            .inner_tree
            .select(&selector)
            .next()
            .ok_or_else(|| anyhow!("Select failed"))
    }

    pub async fn fetch(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.tree.inner_tree = self.get_from_remote().await?;
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

    use crate::slaves::fetchers::{BaseFetcher, FetchItem, HtmlTree};

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
            tree: HtmlTree::default(),
            fetched: vec![],
        };

        fetcher.fetch().await.expect("Fetch failed");

        assert_eq!(fetcher.fetched[0].as_ref().unwrap(), "More information...");
    }
}
