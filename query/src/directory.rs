use async_trait::async_trait;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Directory {
    id: i64,
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
    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}


#[async_trait]
pub trait AppPoolDirectory<Database: sqlx::Database> {
    // async fn get_directory
}

