use std::{fmt::Display, sync::Arc};

use anyhow::Result;

use tokio::sync::mpsc::{self, Receiver, Sender};

#[derive(Clone, Copy)]
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

pub struct TgNotifier {
    tx: Sender<Signal>,
}

impl TgNotifier {
    pub fn new(token: &str) -> Self {
        let (tx, rx) = mpsc::channel(32);
        Self::start(rx);
        Self { tx }
    }

    fn start(mut rx: Receiver<Signal>) {
        tokio::spawn(async move {
            while let Some(signal) = rx.recv().await {
                Self::process_signal(signal).await.unwrap();
            }
        });
    }

    async fn process_signal(signal: Signal) -> Result<()> {
        println!("{}", signal);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Signal::*, TgNotifier};

    use futures::{future::join_all, FutureExt};

    #[tokio::test]
    async fn test_notifier() {
        let notifier = TgNotifier::new("");
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

        join_all(futures).await;
    }
}
