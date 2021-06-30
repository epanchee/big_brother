use std::fmt::Display;

use anyhow::{anyhow, Result};
use scraper::{ElementRef, Html, Selector};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct FetchItem {
    pub name: String,
    pub path: String,
    pub primary: bool,
    pub item_type: String,
    pub related: Vec<FetchItem>,
}

pub trait Seekable<Output = String> {
    fn seek(&self, data: ElementRef) -> Output;
    fn select<'a>(&'a self, tree: &'a Html, path: &str) -> Result<ElementRef> {
        let selector = Selector::parse(path)
            .map_err(|x| anyhow!("Selector parsing errored {:?}", x))?;
        tree.select(&selector)
            .next()
            .ok_or_else(|| anyhow!("Select failed"))
    }
}

impl Seekable for FetchItem {
    fn seek(&self, data: ElementRef) -> String {
        data.inner_html()
    }
}

/// Abstract structure to store fetch item, its fetched content and content of related items
///
/// *fetch_item* - item itself. Should implement Fetchable trait.
/// *content* - a represenation of fetch action (literally select + seek). May be any type (T) but often it is String.
/// *related* - an array of related item fetch results. Their content type may differ from original FoundItem thus I represents inner type.  
///
#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct FoundItem<F: PartialEq + Seekable = FetchItem, T = String, I = T> {
    pub fetch_item: F,
    pub content: T,
    pub related: Vec<Option<FoundItem<F, I>>>,
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
        if let Ok(data) = item.select(&tree, &item.path) {
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

    use crate::slaves::fetchers::{FetchItem, Fetcher};

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
            item_type: "".to_string(),
            related: vec![],
        };

        let fetcher = Fetcher {
            items: vec![item1],
            url: "http://example.com/".to_string(),
        };

        let fetched = fetcher.fetch().await.expect("Fetch failed");

        assert_eq!(fetched[0].as_ref().unwrap().content, "More information...");
    }
}
