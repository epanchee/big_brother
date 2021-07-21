use std::fmt::Display;

use anyhow::Result;

use rutebot::{client::Rutebot, requests::SendMessage, responses::Message};
use tokio::sync::mpsc::{self, Receiver, Sender};

#[derive(Clone, Copy, Debug)]
enum Signal<T = String> {
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

struct RutebotWrapper(Rutebot, String);

pub struct TgNotifier {
    tx: Sender<Signal>,
}

impl TgNotifier {
    pub fn new(token: &str, chat_id: &str) -> Self {
        let (tx, rx) = mpsc::channel(32);
        let bot = RutebotWrapper(Rutebot::new(token), chat_id.to_string());
        let notifier = Self { tx };
        Self::start(rx, bot);
        notifier
    }

    fn start(mut rx: Receiver<Signal>, bot: RutebotWrapper) {
        tokio::spawn(async move {
            while let Some(signal) = rx.recv().await {
                Self::process_signal(signal, &bot).await.unwrap();
            }
        });
    }

    async fn process_signal(signal: Signal, bot: &RutebotWrapper) -> Result<()> {
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
        let tx2 = notifier.tx.clone();
        let tx3 = notifier.tx.clone();

        let futures = vec![
            async move {
                tokio::time::sleep(Duration::from_secs(1)).await;
                notifier.tx.send(Msg("Hello, World!".to_string())).await
            }
            .boxed(),
            tx2.send(Action("You must do it".to_string())).boxed(),
            tx3.send(Err("Crashed".to_string())).boxed(),
        ];

        try_join_all(futures).await.unwrap();
    }
}
