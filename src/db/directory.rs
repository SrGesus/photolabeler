use sqlx::{query, query_as, sqlite::SqliteQueryResult, Pool, Sqlite};

use crate::error::Error;

use super::Database;

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
    pub async fn get_all(Database(pool): &Database) -> Result<Vec<Directory>, Error> {
        Ok(query_as!(Directory, "SELECT * FROM Directory")
            .fetch_all(pool)
            .await?)
    }
    pub async fn get_by_id(Database(pool): &Database, id: i64) -> Result<Directory, Error> {
        Ok(
            query_as!(Directory, "SELECT * FROM Directory WHERE id = ?", id)
                .fetch_one(pool)
                .await?,
        )
    }
    pub async fn get_by_parent_id(
        Database(pool): &Database,
        parent_id: i64,
    ) -> Result<Vec<Directory>, Error> {
        Ok(query_as!(
            Directory,
            "SELECT * FROM Directory WHERE parent_id = ?",
            parent_id
        )
        .fetch_all(pool)
        .await?)
    }
    pub async fn get_by_parentless(Database(pool): &Database) -> Result<Vec<Directory>, Error> {
        Ok(
            query_as!(Directory, "SELECT * FROM Directory WHERE parent_id IS NULL",)
                .fetch_all(pool)
                .await?,
        )
    }
    pub async fn parent_directories(self, database: &Database) -> Result<Vec<Directory>, Error> {
        let mut parent_id = self.parent_id;
        let mut parent_directories = Vec::new();
        while let Some(id) = parent_id {
            let dir = Self::get_by_id(database, id).await?;
            parent_id = dir.parent_id;
            parent_directories.push(dir);
        }
        parent_directories.reverse();
        parent_directories.push(self);
        Ok(parent_directories)
    }
    pub async fn insert(&mut self, Database(pool): &Database) -> Result<SqliteQueryResult, Error> {
        Ok(query!(
            r#"INSERT INTO Directory (parent_id, name, path)
                VALUES (?, ?, ?)
            "#,
            self.parent_id,
            self.name,
            self.path
        )
        .execute(pool)
        .await
        .map(|r| {
            self.id = r.last_insert_rowid();
            r
        })?)
    }
    pub async fn update(&self, Database(pool): &Database) -> Result<SqliteQueryResult, Error> {
        Ok(query!(
            r#"UPDATE Directory
                SET name = ?, path = ?
                WHERE id = ?
            "#,
            self.name,
            self.path,
            self.id
        )
        .execute(pool)
        .await?)
    }
    pub async fn delete(&self, Database(pool): &Database) -> Result<SqliteQueryResult, Error> {
        Ok(query!("DELETE FROM Directory WHERE id = ?", self.id)
            .execute(pool)
            .await?)
    }
}
