use std::path::PathBuf;

use async_trait::async_trait;

use crate::directory::AppPoolDirectory;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Image {
    pub id: i64,
    pub directory_id: i64,
    pub name: String,
    pub notes: String,
}

impl Image {
    pub fn new(directory_id: i64, name: String, notes: String) -> Self {
        Self {
            id: 0,
            directory_id,
            name,
            notes,
        }
    }
}

#[async_trait]
pub trait AppPoolImage: AppPoolDirectory {
    async fn get_image_all(&self) -> Result<Vec<Image>, sqlx::Error>;
    async fn get_image_by_id(&self, id: i64) -> Result<Image, sqlx::Error>;
    async fn get_image_by_directory_id(&self, dir_id: i64) -> Result<Vec<Image>, sqlx::Error>;
    async fn get_image_by_label_id(&self, lab_id: i64) -> Result<Vec<Image>, sqlx::Error>;

    async fn insert_image(&self, image: &mut Image) -> Result<(), sqlx::Error>;

    async fn update_image(&self, image: &Image) -> Result<(), sqlx::Error>;
    async fn update_image_directory_many(
        &self,
        ids: Vec<i64>,
        dir_id: i64,
    ) -> Result<(), sqlx::Error>;

    async fn delete_image_by_id(&self, id: i64) -> Result<(), sqlx::Error>;
    async fn delete_image_by_id_many(&self, ids: Vec<i64>) -> Result<(), sqlx::Error>;
}
