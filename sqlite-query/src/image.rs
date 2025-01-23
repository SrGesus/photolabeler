use crate::{SqliteApp, SqliteAppExecutor};
use futures::future::BoxFuture;
use query::image::{AppImageQueryable, Image};
use sqlx::{Executor, Sqlite};
use std::fmt::Debug;

impl<'k, E> AppImageQueryable<'k> for SqliteApp<E>
where
    SqliteAppExecutor<E>: Executor<'k, Database = Sqlite>,
    E: Debug + Send + Sized + 'k,
{
    fn get_image_all<'e>(self: Box<Self>) -> BoxFuture<'e, Result<Vec<Image>, sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async {
            sqlx::query_as!(Image, "SELECT * FROM Image")
                .fetch_all(self.into_executor())
                .await
        })
    }
    fn get_image_by_id<'e>(self: Box<Self>, id: i64) -> BoxFuture<'e, Result<Image, sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            sqlx::query_as!(Image, "SELECT * FROM Image WHERE id = ?", id)
                .fetch_one(self.into_executor())
                .await
        })
    }
    fn get_image_by_name_in_dir<'e>(
        self: Box<Self>,
        dir_id: i64,
        name: &'e str,
    ) -> BoxFuture<'e, Result<Image, sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            sqlx::query_as!(
                Image,
                r#"SELECT * FROM Image
                    WHERE (directory_id, name) = (?, ?)
                "#,
                dir_id,
                name
            )
            .fetch_one(self.into_executor())
            .await
        })
    }
    fn get_image_by_directory_id<'e>(
        self: Box<Self>,
        dir_id: i64,
    ) -> BoxFuture<'e, Result<Vec<Image>, sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            sqlx::query_as!(Image, "SELECT * FROM Image WHERE directory_id = ?", dir_id)
                .fetch_all(self.into_executor())
                .await
        })
    }
    fn get_image_by_label_id<'e>(
        self: Box<Self>,
        lab_id: i64,
    ) -> BoxFuture<'e, Result<Vec<Image>, sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            sqlx::query_as!(
                Image,
                r#"SELECT i.*
                        FROM Image as i
                        INNER JOIN Labeling ON image_id = i.id
                        WHERE label_id = ?"#,
                lab_id
            )
            .fetch_all(self.into_executor())
            .await
        })
    }
    fn insert_image<'e>(
        self: Box<Self>,
        image: &'e mut Image,
    ) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            sqlx::query!(
                r#"INSERT INTO Image
                    (directory_id, name, notes, created)
                    VALUES (?, ?, ?, ?)"#,
                image.directory_id,
                image.name,
                image.notes,
                image.created
            )
            .execute(self.into_executor())
            .await
            .map(|r| {
                image.id = r.last_insert_rowid();
                r
            })?;
            Ok(())
        })
    }
    fn update_image<'e>(self: Box<Self>, image: &'e Image) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            sqlx::query!(
                r#"UPDATE Image
                    SET directory_id = ?, name = ?, notes = ?
                    WHERE id = ?
                "#,
                image.directory_id,
                image.name,
                image.notes,
                image.id
            )
            .execute(self.into_executor())
            .await?;
            Ok(())
        })
    }
    fn update_image_directory_many<'e>(
        self: Box<Self>,
        ids: &'e Vec<i64>,
        dir_id: i64,
    ) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            let json_ids = sqlx::types::Json(ids);
            sqlx::query!(
                r#"UPDATE Image
                SET directory_id = ?
                WHERE id IN (SELECT value from json_each(?))
            "#,
                dir_id,
                json_ids
            )
            .execute(self.into_executor())
            .await?;
            Ok(())
        })
    }
    fn delete_image_by_id<'e>(self: Box<Self>, id: i64) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            sqlx::query!("DELETE FROM Image WHERE id = ?", id)
                .execute(self.into_executor())
                .await?;
            Ok(())
        })
    }
    fn delete_image_by_id_many<'e>(
        self: Box<Self>,
        ids: &'e Vec<i64>,
    ) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            let json_ids = sqlx::types::Json(ids);
            sqlx::query!(
                "DELETE FROM Image WHERE id IN (SELECT value from json_each(?))",
                json_ids
            )
            .execute(self.into_executor())
            .await?;
            Ok(())
        })
    }
}
