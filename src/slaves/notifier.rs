use std::fmt::Display;

use anyhow::{anyhow, Result};

use rutebot::{client::Rutebot, requests::SendMessage, responses::Message};
use tokio::sync::mpsc::{self, Sender};

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

#[derive(Clone)]
pub struct TgNotifier<T: Display = String> {
    tx: Option<Sender<Signal<T>>>,
    bot: RutebotWrapper,
}

impl<T> TgNotifier<T>
where
    T: Display + Send,
    Signal<T>: Display,
{
    pub fn new(token: &str, chat_id: &str) -> Self {
        let bot = RutebotWrapper(Rutebot::new(token), chat_id.to_string());
        Self { tx: None, bot }
    }

    fn start(&'static mut self) {
        tokio::spawn(async move {
            let (tx, mut rx) = mpsc::channel(32);
            self.tx = Some(tx);
            while let Some(signal) = rx.recv().await {
                Self::process_signal(signal, &self.bot).await.unwrap();
            }
        });
    }

    async fn send(&self, signal: Signal<T>) -> Result<()> {
        self.tx
            .as_ref()
            .ok_or_else(|| anyhow!("No tx avalible"))?
            .send(signal)
            .await
            .map_err(|_| anyhow!("Send error"))
    }

    async fn process_signal(signal: Signal<T>, bot: &RutebotWrapper) -> Result<()> {
        println!("{}", signal);
        /*         bot.0
        .prepare_api_request(SendMessage::new(&bot.1[..], &signal.to_string()[..]))
        .send()
        .await?; */
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
        let notifier = TgNotifier::new("", "");
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
    }
}
