use std::path::PathBuf;

use async_trait::async_trait;
use image::{AppPoolImage, Image};

pub mod directory;
pub mod image;
// pub mod label;

#[async_trait]
pub trait AppPool<Database: sqlx::Database>: Sized + AppPoolImage<Database> {
    async fn new(url: &str) -> Result<Self, sqlx::Error>;
    
    async fn get_image_path(&self, image: &Image) -> Result<PathBuf, sqlx::Error> {
        todo!()
    }
}
