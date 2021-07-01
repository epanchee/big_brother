use std::fmt::Display;

use anyhow::{anyhow, Result};
use scraper::{ElementRef, Html, Selector};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum FetchItemType {
    Class,
    Text,
}
use FetchItemType::*;

#[derive(Debug, Deserialize, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum FoundItemContent {
    Str(String),
    Arr(Vec<String>),
}

use FoundItemContent::*;

#[derive(Debug, Deserialize, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct FetchItem {
    pub name: String,
    pub path: String,
    pub primary: bool,
    pub item_type: FetchItemType,
    pub related: Vec<Self>,
}

impl FetchItem {
    fn seek(&self, data: ElementRef) -> FoundItemContent {
        match self.item_type {
            Class => Arr(data
                .value()
                .to_owned()
                .classes
                .into_iter()
                .map(|x| x.to_string())
                .collect()),
            Text => Str(data.inner_html()),
        }
    }

    fn select<'a>(&'a self, tree: &'a Html) -> Result<ElementRef> {
        let selector =
            Selector::parse(&self.path).map_err(|x| anyhow!("Selector parsing errored {:?}", x))?;
        tree.select(&selector)
            .next()
            .ok_or_else(|| anyhow!("Select failed"))
    }
}

#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct FoundItem {
    pub fetch_item: FetchItem,
    pub content: FoundItemContent,
    pub related: Vec<Option<FoundItem>>,
}

#[derive(Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Fetcher {
    pub items: Vec<FetchItem>,
    pub url: String,
}

impl Fetcher {
    async fn get_from_remote(&self) -> Result<Html> {
        let resp_text = reqwest::get(&self.url).await?.text().await?;
        Ok(Html::parse_document(&resp_text[..]))
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

    pub async fn fetch(&self) -> Result<Vec<Option<FoundItem>>> {
        let tree = self.get_from_remote().await?;
        let mut fetched = vec![];
        let primary_items: Vec<_> = self.items.iter().filter(|&item| item.primary).collect();
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
}

impl Display for Fetcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fetcher: url={}", self.url)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use scraper::{Html, Selector};

    use crate::slaves::fetchers::{FetchItem, FetchItemType, Fetcher, FoundItemContent};

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
            primary: true,
            item_type: FetchItemType::Text,
            related: vec![],
        };

        let fetcher = Fetcher {
            items: vec![item1],
            url: "http://example.com/".to_string(),
        };

        let fetched = fetcher.fetch().await.expect("Fetch failed");

        assert_eq!(
            fetched[0].as_ref().unwrap().content,
            FoundItemContent::Str("More information...".to_string())
        );
    }
}
