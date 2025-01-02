use std::path::PathBuf;

use directory::AppDirectoryQueryable;
use futures::future::BoxFuture;
use image::{AppImageQueryable, Image};
use label::AppLabelQueryable;

pub mod directory;
pub mod executor;
pub mod image;
pub mod label;

pub trait AppQueryable<'k>:
    AppImageQueryable<'k> + AppDirectoryQueryable<'k> + AppLabelQueryable<'k>
{
    fn get_image_path<'e>(self: Box<Self>, image: &'e Image) -> BoxFuture<'e, Result<PathBuf, sqlx::Error>>
    where
        'k: 'e;
}

pub trait AppPool {
    fn transaction<'e>(
        &'e self,
    ) -> BoxFuture<'e, Result<Box<dyn AppTransaction<'e> + 'e>, sqlx::Error>>;
    fn queryable<'k>(&'k self) -> Box<dyn AppQueryable<'k> + 'k>;
}

pub trait AppTransaction<'k> {
    fn commit<'e>(self) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e;
    fn rollback<'e>(self) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e;
    fn queryable<'e>(&'e mut self) -> BoxFuture<'e, Box<dyn AppQueryable<'e> + 'e>>;
}
