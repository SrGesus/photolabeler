use std::path::{Path, PathBuf};

use directory::AppDirectoryQueryable;
use futures::future::BoxFuture;
use image::{AppImageQueryable, Image};
use label::AppLabelQueryable;
use std::fmt::Debug;

pub mod directory;
pub mod executor;
pub mod image;
pub mod label;

pub trait AppQueryable<'k>:
    Send + 'k + AppImageQueryable<'k> + AppDirectoryQueryable<'k> + AppLabelQueryable<'k>
{
    fn get_image_path<'e>(
        self: Box<Self>,
        image: &'e Image,
    ) -> BoxFuture<'e, Result<PathBuf, sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            Ok(
                Path::new(&self.get_directory_by_id(image.directory_id).await?.path)
                    .join(&image.name),
            )
        })
    }
}

impl<'k, E> AppQueryable<'k> for E where
    E: AppImageQueryable<'k>
        + AppDirectoryQueryable<'k>
        + AppLabelQueryable<'k>
        + Debug
        + Send
        + Sized
        + 'k
{
}

pub trait AppPool {
    fn transaction<'e>(
        &'e self,
    ) -> BoxFuture<'e, Result<Box<dyn AppTransaction<'e> + 'e>, sqlx::Error>>;
    fn queryable<'k>(&'k self) -> Box<dyn AppQueryable<'k> + 'k>;
}

pub trait AppTransaction<'k> {
    fn commit<'e>(self: Box<Self>) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e;
    fn rollback<'e>(self: Box<Self>) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e;
    fn queryable<'e>(&'e mut self) ->  Box<dyn AppQueryable<'e> + 'e>;
}
