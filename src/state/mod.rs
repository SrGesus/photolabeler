use std::{path, sync::Arc};

use futures::{stream::FuturesUnordered, StreamExt};
use query::{directory::Directory, image::Image, AppPool};
use sqlite_query::SqliteAppPool;
use tokio::fs::{self, DirEntry};

use crate::error::Error;

pub mod directory;
pub mod image;
pub mod label;

#[derive(Clone)]
pub struct AppState {
    pool: Arc<Box<dyn AppPool>>,
}

impl AppState {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let database = &database_url[..database_url.find(':').unwrap_or(database_url.len())];

        let pool = match database {
            "sqlite" => {
                Arc::new(Box::new(SqliteAppPool::new(database_url).await?) as Box<dyn AppPool>)
            }
            _ => {
                tracing::error!("Could not recognize database type: {database}");
                panic!();
            }
        };

        Ok(Self { pool })
    }

    async fn remove_missing_directory(&self, dir: Directory) -> Result<(), Error> {
        if fs::metadata(dir.path).await.is_err() {
            self.pool.queryable().delete_directory(dir.id).await?;
        }
        Ok(())
    }

    async fn remove_missing_image(&self, path: &path::Path, image: Image) -> Result<(), Error> {
        if fs::metadata(path.join(&image.name)).await.is_err() {
            self.pool.queryable().delete_image_by_id(image.id).await?;
        }
        Ok(())
    }

    pub async fn delete_missing(&self, dir: &Directory) -> Result<(), Error> {
        // Directories in directory
        let directories = self.get_directory_by_parent_id(dir.id).await?;
        let dir_path = path::Path::new(&dir.path);

        let mut dir_futures = directories
            .into_iter()
            .map(|dir| self.remove_missing_directory(dir))
            .collect::<FuturesUnordered<_>>();
        while let Some(res) = dir_futures.next().await {
            res?;
        }

        // Images in directory
        let images = self.get_image_by_directory_id(dir.id).await?;

        let mut im_futures = images
            .into_iter()
            .map(|im| self.remove_missing_image(dir_path, im))
            .collect::<FuturesUnordered<_>>();
        while let Some(res) = im_futures.next().await {
            res?;
        }
        Ok(())
    }

    async fn add_missing_file(
        &self,
        dir: &Directory,
        file: DirEntry,
    ) -> Result<Option<Directory>, Error> {
        let mut file_type = file.file_type().await?;
        let mut file_path = file.path();
        let name = file.file_name().into_string().unwrap();

        // Follow symlinks
        while file_type.is_symlink() {
            file_path = fs::read_link(file_path).await?;
            file_type = fs::metadata(&file_path).await?.file_type();
        }

        if file_type.is_dir() {
            let mut new_dir = Directory::new(
                Some(dir.id),
                name,
                file.path().into_os_string().into_string().unwrap(),
            );
            if let Ok(_) = self.pool.queryable().insert_directory(&mut new_dir).await {
                return Ok(Some(new_dir));
            }
        } else if file_type.is_file()
            && mime_guess::from_path(file.path())
                .first_raw()
                .is_some_and(|content| content.contains("image"))
        {
            let mut i = Image::new(dir.id, name, String::new());
            // Ignore errors on insert if image is already present on the database
            self.pool.queryable().insert_image(&mut i).await.ok();
        }
        Ok(None)
    }

    pub async fn add_missing(&self, directory: Directory) -> Result<(), Error> {
        let mut v = vec![directory];
        while let Some(dir) = v.pop() {
            if fs::metadata(&dir.path).await.is_err() {
                continue;
            }

            // Insert files from directory into database
            if let Ok(mut entries) = fs::read_dir(&dir.path).await {
                let mut files = Vec::new();
                while let Some(file) = entries.next_entry().await.unwrap() {
                    files.push(file);
                }

                let mut futures = files
                    .into_iter()
                    .map(|file| self.add_missing_file(&dir, file))
                    .collect::<FuturesUnordered<_>>();

                while let Some(res) = futures.next().await {
                    if let Ok(Some(new_dir)) = res {
                        v.push(new_dir);
                    }
                }
            }
        }
        Ok(())
    }
}
