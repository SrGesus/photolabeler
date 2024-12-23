use std::path::Path;

pub(crate) mod image;
pub(crate) mod directory;

use sqlx::{
    migrate,
    migrate::{Migrate, Migrator},
    sqlite::SqliteArguments,
    Connection, Executor, Pool, Sqlite, SqliteConnection, SqlitePool,
};

pub struct Database(pub(crate) Pool<Sqlite>);

impl Database {
    pub async fn new(url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(url).await?;

        // Run migrations that on compile-time are in './migrations'
        migrate!("./migrations").run(&pool).await?;

        Ok(Self(pool))
    }

}
