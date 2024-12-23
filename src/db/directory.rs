use sqlx::{query, query_as, sqlite::SqliteQueryResult, Error, Pool, Sqlite};

use super::Database;

pub struct Directory {
    id: i64,
    name: String,
    path: String,
}

impl Directory {
    pub fn new(name: String, path: String) -> Self {
        Self { id: 0, name, path }
    }
    pub fn id(&self) -> &i64 {
        &self.id
    }
    pub fn path(&self) -> &String {
        &self.path
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub async fn get_all(Database(pool): &Database) -> Result<Vec<Directory>, Error> {
        query_as!(Directory, "SELECT * FROM Directory")
            .fetch_all(pool)
            .await
    }
    pub async fn insert(&self, Database(pool): &Database) -> Result<SqliteQueryResult, Error> {
        query!(
            "INSERT INTO Directory (name, path) VALUES (?, ?)",
            self.name,
            self.path
        )
        .execute(pool)
        .await
    }
    pub async fn delete(&self, Database(pool): &Database) -> Result<SqliteQueryResult, Error> {
        query("DELETE FROM Directory WHERE id = ?")
            .bind(self.id)
            .execute(pool)
            .await
    }
    pub async fn update(&self, Database(pool): &Database) -> Result<SqliteQueryResult, Error> {
        query!(
            r#"UPDATE Directory
                SET name = ?, path = ?
                WHERE id = ?
            "#,
            self.name,
            self.path,
            self.id
        )
        .bind(self.id)
        .execute(pool)
        .await
    }
}
