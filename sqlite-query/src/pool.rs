use futures::future::BoxFuture;
use query::{AppPool, AppQueryable, AppTransaction};
use sqlx::{migrate, Pool, Sqlite};

use crate::SqliteApp;

pub struct SqliteAppPool(Pool<Sqlite>);

impl SqliteAppPool {
    pub async fn new(url: &str) -> Result<Self, sqlx::Error> {
        let pool = <Pool<Sqlite>>::connect(url).await?;

        // Run migrations that on compile-time are in '../migrations'
        migrate!("../migrations").run(&pool).await?;

        Ok(Self(pool))
    }
}

impl Clone for SqliteAppPool {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl AppPool for SqliteAppPool {
    fn transaction<'e>(
        &'e self,
    ) -> BoxFuture<'e, Result<Box<dyn AppTransaction<'e> + 'e>, sqlx::Error>> {
        Box::pin(async move {
            Ok(Box::new(SqliteApp(self.0.begin().await?)) as Box<dyn AppTransaction<'e>>)
        })
    }
    fn queryable<'k>(&'k self) -> Box<dyn AppQueryable<'k> + 'k> {
        Box::new(SqliteApp(&self.0)) as Box<dyn AppQueryable<'k>>
    }
}
