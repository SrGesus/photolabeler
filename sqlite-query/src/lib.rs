use itertools::Itertools;
use std::path::PathBuf; // 0.8.0

use async_trait::async_trait;
use query::{
    image::{AppPoolImage, Image},
    AppPool,
};
use sqlx::{migrate, sqlite::SqliteQueryResult, Pool, QueryBuilder, Sqlite, SqlitePool};
pub struct SqliteAppPool(Pool<Sqlite>);

#[async_trait]
impl AppPool<Sqlite> for SqliteAppPool {
    async fn new(url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(url).await?;

        // Run migrations that on compile-time are in '../migrations'
        migrate!("../migrations").run(&pool).await?;

        Ok(Self(pool))
    }
}

#[async_trait]
impl AppPoolImage<Sqlite> for SqliteAppPool {
    async fn get_image_all(&self) -> Result<Vec<Image>, sqlx::Error> {
        Ok(sqlx::query_as!(Image, "SELECT * FROM Image")
            .fetch_all(&self.0)
            .await?)
    }

    async fn get_image_by_id(&self, id: i64) -> Result<Image, sqlx::Error> {
        Ok(
            sqlx::query_as!(Image, "SELECT * FROM Image WHERE id = ?", id)
                .fetch_one(&self.0)
                .await?,
        )
    }

    async fn get_image_by_directory_id(&self, id: i64) -> Result<Vec<Image>, sqlx::Error> {
        Ok(
            sqlx::query_as!(Image, "SELECT * FROM Image WHERE directory_id = ?", id)
                .fetch_all(&self.0)
                .await?,
        )
    }

    async fn get_image_by_label_id(&self, id: i64) -> Result<Vec<Image>, sqlx::Error> {
        Ok(sqlx::query_as!(
            Image,
            r#"SELECT id, directory_id, name, notes
                    FROM Image
                    INNER JOIN Labeling ON image_id = id
                    WHERE label_id = ?"#,
            id
        )
        .fetch_all(&self.0)
        .await?)
    }

    async fn insert_image(&self, image: &mut Image) -> Result<SqliteQueryResult, sqlx::Error> {
        Ok(sqlx::query!(
            r#"INSERT INTO Image
            (directory_id, name, notes)
            VALUES (?, ?, ?)"#,
            image.directory_id,
            image.name,
            image.notes
        )
        .execute(&self.0)
        .await
        .map(|r| {
            image.id = r.last_insert_rowid();
            r
        })?)
    }

    async fn update_image(&self, image: &Image) -> Result<SqliteQueryResult, sqlx::Error> {
        Ok(sqlx::query!(
            r#"UPDATE Image
                SET directory_id = ?, name = ?, notes = ?
                WHERE id = ?
            "#,
            image.directory_id,
            image.name,
            image.notes,
            image.id
        )
        .execute(&self.0)
        .await?)
    }
    async fn delete_image_by_id(&self, id: i64) -> Result<SqliteQueryResult, sqlx::Error> {
        Ok(sqlx::query!("DELETE FROM Image WHERE id = ?", id)
            .execute(&self.0)
            .await?)
    }
    async fn delete_image_by_ids(&self, ids: Vec<i64>) -> Result<SqliteQueryResult, sqlx::Error> {
        let ids_str: String =
            itertools::Itertools::intersperse(ids.iter().map(|i| i.to_string()), ", ".to_string())
                .collect();
        Ok(
            sqlx::query(&format!("DELETE FROM Image WHERE id = ({ids_str})"))
                .execute(&self.0)
                .await?,
        )
    }
}
