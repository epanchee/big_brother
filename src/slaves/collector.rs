use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use anyhow::{Error, Result};
use sqlx::types::Json;

use super::fetchers::FoundItem;

#[derive(Clone)]
pub struct PgCollector {
    pool: Pool<Postgres>,
}

impl PgCollector {
    pub async fn new() -> Result<Self, Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&dotenv::var("DATABASE_URL")?)
            .await?;

        Ok(PgCollector { pool })
    }

    pub async fn store(&self, data: Vec<Vec<FoundItem>>) -> Result<()> {
        sqlx::query!(
            r#"
INSERT INTO history (data)
VALUES ( $1 )
            "#,
            Json(data) as _
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
