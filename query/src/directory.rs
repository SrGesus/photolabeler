use async_trait::async_trait;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Directory {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub path: String,
}

impl Directory {
    pub fn new(parent_id: Option<i64>, name: String, path: String) -> Self {
        Self {
            id: 0,
            parent_id,
            name,
            path,
        }
    }
}

#[async_trait]
pub trait AppPoolDirectory {
    async fn get_directory_all(&self) -> Result<Vec<Directory>, sqlx::Error>;
    async fn get_directory_by_id(&self, id: i64) -> Result<Directory, sqlx::Error>;
    async fn get_directory_by_parent_id(&self, par_id: i64) -> Result<Vec<Directory>, sqlx::Error>;
    async fn get_directory_parentless(&self) -> Result<Vec<Directory>, sqlx::Error>;

    async fn directory_ancestors(&self, dir: &Directory) -> Result<Vec<Directory>, sqlx::Error>;

    async fn insert_directory(&self, dir: &mut Directory) -> Result<(), sqlx::Error>;

    async fn update_directory(&self, dir: &Directory) -> Result<(), sqlx::Error>;

    async fn delete_directory(&self, id: i64) -> Result<(), sqlx::Error>;
}
