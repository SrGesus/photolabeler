use futures::{future::BoxFuture, FutureExt};
use sqlx::{Sqlite, Transaction};
use std::{fmt::Debug, path::Path, path::PathBuf};

use query::{
    directory::AppDirectoryQueryable,
    executor::AppExecutor,
    image::{AppImageQueryable, Image},
    label::AppLabelQueryable, AppQueryable, AppTransaction,
};

pub mod pool;
pub mod directory;
pub mod image;
pub mod label;

pub use pool::*;

type SqliteAppExecutor<E> = AppExecutor<E, Sqlite>;

#[derive(Debug)]
pub struct SqliteApp<E: Debug>(E);


impl<E: Debug + Sized + Send> SqliteApp<E> {
    fn into_executor(self) -> SqliteAppExecutor<E> {
        SqliteAppExecutor::new(self.0)
    }
}

pub type SqliteTransaction<'k> = SqliteApp<Transaction<'k, Sqlite>>;

impl<'k> AppTransaction<'k> for SqliteTransaction<'k> {
    fn commit<'e>(self: Box<Self>) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e,
    {
        self.0.commit().boxed()
    }

    fn rollback<'e>(self: Box<Self>) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e,
    {
        self.0.rollback().boxed()
    }

    fn queryable<'e>(&'e mut self) -> Box<dyn AppQueryable<'e> + 'e> {
        Box::new(SqliteApp(&mut *self.0)) as Box<dyn AppQueryable<'e>>
    }
}
