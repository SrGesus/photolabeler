use std::path::{Path, PathBuf};

use async_trait::async_trait;
use directory::AppPoolDirectory;
use image::{AppPoolImage, Image};
use label::AppPoolLabel;

pub mod directory;
pub mod image;
pub mod label;

#[async_trait]
pub trait AppPool: AppPoolImage + AppPoolDirectory + AppPoolLabel {
    async fn get_image_path(&self, image: &Image) -> Result<PathBuf, sqlx::Error> {
        Ok(Path::new(&self.get_directory_by_id(image.directory_id).await?.path).join(&image.name))
    }
}
