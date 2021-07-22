use std::{fmt::Display, fs};

use anyhow::{anyhow, Result};

use rutebot::{client::Rutebot, requests::SendMessage};
use serde::Deserialize;
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};

#[derive(Clone, Copy, Debug)]
pub enum Signal<T: Display = String> {
    Action(T),
    Msg(T),
    Err(T),
}

impl Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Signal::Action(msg) => write!(f, "Action required: {}", msg),
            Signal::Msg(msg) => write!(f, "{}", msg),
            Signal::Err(msg) => write!(f, "Error occured: {}", msg),
        }
    }
}

#[derive(Clone)]
struct RutebotWrapper(Rutebot, String);

#[derive(Deserialize)]
pub struct TgConfig {
    token: String,
    chat_id: String,
}

#[derive(Clone, Debug)]
pub struct TgNotifier<T: Display + Send = String> {
    tx: Sender<Signal<T>>,
}

impl<T> TgNotifier<T>
where
    T: Display + Send + 'static,
    Signal<T>: Display,
{
    pub fn new() -> Result<Self> {
        Ok(Self::new_with_loop_handle()?.0)
    }

    pub fn new_with_loop_handle() -> Result<(Self, JoinHandle<()>)> {
        let (tx, rx) = mpsc::channel(32);
        let bot = Self::create_bot_from_conf()?;
        Ok((Self { tx }, Self::create_channel_loop(rx, bot)))
    }

    fn create_bot_from_conf() -> Result<RutebotWrapper> {
        let content = fs::read_to_string("config/tg.yaml")?;
        let conf: TgConfig = serde_yaml::from_str(&content)?;
        Ok(RutebotWrapper(Rutebot::new(conf.token), conf.chat_id))
    }

    fn create_channel_loop(mut rx: Receiver<Signal<T>>, bot: RutebotWrapper) -> JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(signal) = rx.recv().await {
                Self::process_signal(signal, &bot).await.unwrap();
            }
        })
    }

    pub async fn send(&self, signal: Signal<T>) -> Result<()> {
        self.tx
            .send(signal)
            .await
            .map_err(|_| anyhow!("Send error"))
    }

    async fn process_signal(signal: Signal<T>, bot: &RutebotWrapper) -> Result<()> {
        println!("{}", signal);
        let msg = &signal.to_string()[..];
        bot.0
            .prepare_api_request(SendMessage::new(&bot.1[..], msg))
            .send()
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Signal::*, TgNotifier};

    use futures::{future::try_join_all, FutureExt};

    #[tokio::test]
    async fn test_notifier() {
        let (notifier, loop_h) = TgNotifier::new_with_loop_handle().unwrap();
        let notifier2 = notifier.clone();

        let futures = vec![
            async move {
                tokio::time::sleep(Duration::from_secs(1)).await;
                notifier2.send(Msg("Hello, World!".to_string())).await
            }
            .boxed(),
            notifier.send(Action("You must do it".to_string())).boxed(),
            notifier.send(Err("Crashed".to_string())).boxed(),
        ];

        try_join_all(futures).await.unwrap();
        drop(notifier);
        loop_h.await.unwrap();
    }
}
