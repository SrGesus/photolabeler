use std::path::PathBuf;

use async_trait::async_trait;

use crate::directory::Directory;

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
    pub fn id(&self) -> &i64 {
        &self.id
    }
    pub fn directory_id(&self) -> &i64 {
        &self.directory_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn notes(&self) -> &str {
        &self.notes
    }
}

#[async_trait]
pub trait AppPoolImage<Database: sqlx::Database> {
    async fn get_image_all(&self) -> Result<Vec<Image>, sqlx::Error>;
    async fn get_image_by_id(&self, id: i64) -> Result<Image, sqlx::Error>;
    async fn get_image_by_directory_id(&self, id: i64) -> Result<Vec<Image>, sqlx::Error>;
    async fn get_image_by_label_id(&self, id: i64) -> Result<Vec<Image>, sqlx::Error>;

    async fn insert_image(&self, image: &mut Image) -> Result<Database::QueryResult, sqlx::Error>;
    async fn update_image(&self, image: &Image) -> Result<Database::QueryResult, sqlx::Error>;
    async fn delete_image_by_id(&self, id: i64) -> Result<Database::QueryResult, sqlx::Error>;
    async fn delete_image_by_ids(&self, ids: Vec<i64>) -> Result<Database::QueryResult, sqlx::Error>;

}
