use std::sync::Arc;

use tokio_postgres::{Client, Error, NoTls};

#[derive(Clone, Debug)]
pub struct PgCollector {
    client: Arc<Client>,
}

impl PgCollector {
    pub async fn new() -> Result<Self, Error> {
        let (client, connection) =
            tokio_postgres::connect("host=localhost user=postgres", NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Database connection error: {}", e);
            }
        });

        Ok(PgCollector {
            client: Arc::new(client),
        })
    }
}
