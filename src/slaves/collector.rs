use std::sync::Arc;

use tokio_postgres::{Client, Error, NoTls, Statement};

use anyhow::Result;

#[derive(Clone)]
pub struct PgCollector {
    client: Arc<Client>,
    save_query: Statement,
}

impl PgCollector {
    pub async fn new() -> Result<Self, Error> {
        let (client, connection) =
            tokio_postgres::connect("host=localhost user=postgres password=password", NoTls)
                .await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Database connection error: {}", e);
            }
        });

        client
            .query(
                "CREATE TABLE history
                    (
                        id SERIAL PRIMARY KEY,
                        data VARCHAR(1024)  NOT NULL,
                        ts TIMESTAMP DEFAULT NOW()
                    )",
                &[],
            )
            .await;

        Ok(PgCollector {
            save_query: client
                .prepare("INSERT INTO history (data) VALUES ($1)")
                .await?,
            client: Arc::new(client),
        })
    }

    pub async fn store(&self, data: String) -> Result<()> {
        self.client.query(&self.save_query, &[&data]).await?;
        Ok(())
    }
}
