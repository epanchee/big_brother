use std::fmt::Display;

use anyhow::{anyhow, Result};
use scraper::{ElementRef, Html, Selector};
use serde::Deserialize;

pub type Config = Vec<FetchItem>;

#[derive(Debug, Deserialize, Clone, PartialEq, PartialOrd, Ord, Eq)]
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

#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct FoundItem<T> {
    pub fetch_item: FetchItem,
    pub content: T,
}

#[derive(Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct BaseFetcher {
    pub items: Config,
    pub url: String,
}

impl BaseFetcher {
    async fn get_from_remote(&self) -> Result<Html, Box<dyn std::error::Error>> {
        let resp_text = reqwest::get(&self.url).await?.text().await?;
        Ok(Html::parse_document(&resp_text[..]))
    }

    fn select<'a, 'b>(selector: &'b str, tree: &'a Html) -> Result<ElementRef<'a>> {
        let selector =
            Selector::parse(selector).map_err(|x| anyhow!("Selector parsing errored {:?}", x))?;
        tree.select(&selector)
            .next()
            .ok_or_else(|| anyhow!("Select failed"))
    }

    pub async fn fetch(&self) -> Result<Vec<Option<FoundItem<String>>>, Box<dyn std::error::Error>> {
        let tree = self.get_from_remote().await?;
        let mut fetched = vec![];
        for root_item in self.items.iter() {
            for item in root_item.iter() {
                fetched.push({
                    if let Ok(data) = Self::select(&item.path, &tree) {
                        Some(FoundItem {
                            fetch_item: item.clone(),
                            content: item.seek(data),
                        })
                    } else {
                        None
                    }
                })
            }
        }
        Ok(fetched)
    }
}

impl Display for BaseFetcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BaseFetcher: url={}", self.url)
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

        let fetcher = BaseFetcher {
            items: vec![item1],
            url: "http://example.com/".to_string(),
        };

        let fetched = fetcher.fetch().await.expect("Fetch failed");

        assert_eq!(fetched[0].as_ref().unwrap().content, "More information...");
    }
}
