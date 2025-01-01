use sqlx::{migrate, Pool, Sqlite, SqlitePool};

struct AppState {
    pool: Pool<Sqlite>,
}

impl AppState {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(database_url).await?;

        // Run migrations that on compile-time are in './migrations'
        migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }
}
