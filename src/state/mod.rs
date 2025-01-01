use query::{directory::AppPoolDirectory, image::AppPoolImage, label::AppPoolLabel, AppPool};
use sqlite_query::SqliteAppPool;

struct AppState {
    pool: Box<dyn AppPool>,
}

impl AppState {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let database = &database_url[..database_url.find(':').unwrap_or(database_url.len())];

        let pool = match database {
            "sqlite" => Box::new(SqliteAppPool::new(database_url).await?),
            _ => {
                tracing::error!("Could not recognize database type: {database}");
                panic!();
            }
        };

        Ok(Self { pool })
    }
}
