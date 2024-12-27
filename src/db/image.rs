use std::path::{Path, PathBuf};

use sqlx::{query, query_as, sqlite::SqliteQueryResult, QueryBuilder};

use crate::error::Error;

use self::directory::Directory;

use super::{directory, Database};

const BIND_LIMIT: usize = 32766;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Image {
    id: i64,
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
    pub async fn path(&self, database: &Database) -> Result<PathBuf, Error> {
        Ok(Path::new(
            Directory::get_by_id(database, self.directory_id)
                .await?
                .path(),
        )
        .join(self.name()))
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
    pub async fn insert_ignore(Database(pool): &Database, images: Vec<Image>) {
        let mut query_builder =
            QueryBuilder::new("INSERT OR IGNORE INTO Image (directory_id, name, notes) ");

        for images_iter in images.chunks(BIND_LIMIT / 4) {
            query_builder.push_values(images_iter, |mut b, im| {
                b.push_bind(im.directory_id)
                    .push_bind(im.name.clone())
                    .push_bind(im.notes.clone());
            });
            query_builder.build().execute(pool).await.ok();
        }
    }
    pub async fn insert(&mut self, Database(pool): &Database) -> Result<SqliteQueryResult, Error> {
        Ok(query!(
            r#"INSERT INTO Image
            (directory_id, name, notes)
            VALUES (?, ?, ?)"#,
            self.directory_id,
            self.name,
            self.notes
        )
        .execute(pool)
        .await
        .map(|r| {
            self.id = r.last_insert_rowid();
            r
        })?)
    }
    pub async fn delete(&self, database: &Database) -> Result<SqliteQueryResult, Error> {
        Self::delete_by_id(database, self.id).await
    }
    pub async fn delete_by_id(
        Database(pool): &Database,
        image_id: i64,
    ) -> Result<SqliteQueryResult, Error> {
        Ok(query!("DELETE FROM Image WHERE id = ?", image_id)
            .execute(pool)
            .await?)
    }
    pub async fn update(&self, Database(pool): &Database) -> Result<SqliteQueryResult, Error> {
        Ok(query!(
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
        .await?)
    }
    pub async fn get_by_id(Database(pool): &Database, image_id: i64) -> Result<Image, Error> {
        Ok(
            query_as!(Image, "SELECT * FROM Image WHERE id = ?", image_id)
                .fetch_one(pool)
                .await?,
        )
    }
    pub async fn get_all(Database(pool): &Database) -> Result<Vec<Image>, Error> {
        Ok(query_as!(Image, "SELECT * FROM Image",)
            .fetch_all(pool)
            .await?)
    }
    pub async fn get_by_directory(
        Database(pool): &Database,
        directory_id: i64,
    ) -> Result<Vec<Image>, Error> {
        Ok(query_as!(
            Image,
            r#"SELECT i.id, i.directory_id, i.name, i.notes
                FROM Image as i
                INNER JOIN Directory as d ON i.directory_id = d.id
                WHERE d.id = ?"#,
            directory_id
        )
        .fetch_all(pool)
        .await?)
    }
    pub async fn get_by_label(
        Database(pool): &Database,
        label_id: i64,
    ) -> Result<Vec<Image>, Error> {
        Ok(query_as!(
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
        .await?)
    }
}
