use std::path::Path;

pub(crate) mod directory;
pub(crate) mod image;
pub(crate) mod label;

use image::Image;
use sqlx::{
    migrate,
    migrate::{Migrate, Migrator},
    sqlite::SqliteArguments,
    Connection, Error, Executor, Pool, Sqlite, SqliteConnection, SqlitePool,
};

#[derive(Clone)]
pub struct Database(pub(crate) Pool<Sqlite>);

impl Database {
    pub async fn new(url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(url).await?;

        // Run migrations that on compile-time are in './migrations'
        migrate!("./migrations").run(&pool).await?;

        Ok(Self(pool))
    }
}
