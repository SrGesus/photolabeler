use sqlx::{query, query_as, sqlite::SqliteQueryResult, Error};

use super::Database;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Image {
    id: i64,
    directory_id: i64,
    name: String,
    notes: Option<String>,
}

impl Image {
    pub fn new(directory_id: i64, name: String, notes: Option<String>) -> Self {
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
    pub fn notes(&self) -> &Option<String> {
        &self.notes
    }
    pub async fn insert(self, Database(pool): &Database) -> Result<SqliteQueryResult, Error> {
        query!(
            r#"INSERT INTO Image
            (directory_id, name, notes)
            VALUES (?, ?, ?)"#,
            self.directory_id,
            self.name,
            self.notes
        )
        .execute(pool)
        .await
    }

    pub async fn delete(&self, Database(pool): &Database) -> Result<SqliteQueryResult, Error> {
        query("DELETE FROM Image WHERE id = ?")
            .bind(self.id)
            .execute(pool)
            .await
    }

    pub async fn update(&self, Database(pool): &Database) -> Result<SqliteQueryResult, Error> {
        query!(
            r#"UPDATE Image
                SET directory_id = ?, name = ?, notes = ?
                WHERE id = ?
            "#,
            self.directory_id,
            self.name,
            self.notes,
            self.id
        )
        .execute(pool)
        .await
    }

    pub async fn get_by_id(
        Database(pool): &Database,
        image_id: i64,
    ) -> Result<Option<Image>, Error> {
        query_as!(Image, "SELECT * FROM Image WHERE id = ?", image_id)
            .fetch_optional(pool)
            .await
    }

    pub async fn get_all(Database(pool): &Database) -> Result<Vec<Image>, Error> {
        query_as!(Image, "SELECT * FROM Image",)
            .fetch_all(pool)
            .await
    }

    pub async fn get_by_directory(
        Database(pool): &Database,
        directory_id: i64,
    ) -> Result<Vec<Image>, Error> {
        query_as!(
            Image,
            r#"SELECT i.id, i.directory_id, i.name, i.notes
                FROM Image as i
                INNER JOIN Directory as d ON i.directory_id = d.id
                WHERE d.id = ?"#,
            directory_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn get_by_label(
        Database(pool): &Database,
        label_id: i64,
    ) -> Result<Vec<Image>, Error> {
        query_as!(
            Image,
            r#"SELECT i.id, i.name, notes, i.directory_id as directory_id
                FROM Image as i
                INNER JOIN Directory as d ON i.directory_id = d.id
                INNER JOIN Labeling as l ON l.image_id = i.id
                WHERE l.label_id == ?
            "#,
            label_id
        )
        .fetch_all(pool)
        .await
    }
}
