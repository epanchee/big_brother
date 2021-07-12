use reqwest::{header::HeaderValue, Url};

use anyhow::{anyhow, Result};
use std::{fs::File, io::BufReader, path::Path, sync::RwLock};

use bytes::Bytes;

pub struct Cookie<'a>(pub cookie::Cookie<'a>);

impl<'a> Cookie<'a> {
    fn parse(value: &'a HeaderValue) -> Result<Cookie<'a>> {
        std::str::from_utf8(value.as_bytes())
            .map_err(cookie::ParseError::from)
            .and_then(cookie::Cookie::parse)
            .map(Cookie)
            .map_err(|_| anyhow!("Couldn't parse cookie"))
    }
}

#[derive(Debug)]
struct CookiePath(String);

impl Default for CookiePath {
    fn default() -> Self {
        Self("/tmp/cookies".to_string())
    }
}

#[derive(Debug, Default)]
pub struct MyJar(pub RwLock<cookie_store::CookieStore>, CookiePath);

impl reqwest::cookie::CookieStore for MyJar {
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, url: &Url) {
        let iter =
            cookie_headers.filter_map(|val| Cookie::parse(val).map(|c| c.0.into_owned()).ok());

        self.0.write().unwrap().store_response_cookies(iter, url);
    }

    fn cookies(&self, url: &Url) -> Option<HeaderValue> {
        let s = self
            .0
            .read()
            .unwrap()
            .get_request_cookies(url)
            .map(|c| format!("{}={}", c.name(), c.value()))
            .collect::<Vec<_>>()
            .join("; ");

        if s.is_empty() {
            return None;
        }

        HeaderValue::from_maybe_shared(Bytes::from(s)).ok()
    }
}

impl MyJar {
    pub fn new(cookies_file: String) -> Self {
        let cookies_path = format!("cookies/{}", cookies_file);
        let cookies = if Path::new(&cookies_path[..]).exists() {
            let f = BufReader::new(File::open(&cookies_path).unwrap());
            Self(
                RwLock::new(cookie_store::CookieStore::load_json(f).unwrap()),
                CookiePath(cookies_path),
            )
        } else {
            Self(
                RwLock::new(cookie_store::CookieStore::default()),
                CookiePath(cookies_path),
            )
        };
        cookies
    }

    pub fn store_cookies(&self) -> Result<()> {
        let mut buffer = File::create(&self.1.0)?;
        self.0
            .read()
            .unwrap()
            .save_json(&mut buffer)
            .map_err(|_| anyhow!("Couldn't store cookies"))?;
        Ok(())
    }
}
