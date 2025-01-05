use crate::{SqliteApp, SqliteAppExecutor};
use futures::future::BoxFuture;
use query::label::{AppLabelQueryable, Label};
use sqlx::{Executor, Sqlite};
use std::fmt::Debug;

impl<'k, E> AppLabelQueryable<'k> for SqliteApp<E>
where
    SqliteAppExecutor<E>: Executor<'k, Database = Sqlite>,
    E: Debug + Send + Sized + 'k,
{
    fn get_label_by_image_id<'e>(
        self: Box<Self>,
        image_id: i64,
    ) -> BoxFuture<'e, Result<Vec<Label>, sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            sqlx::query_as!(
                Label,
                r#"SELECT l.*
                FROM Label as l
                INNER JOIN Labeling ON label_id = l.id
                WHERE image_id = ?
            "#,
                image_id
            )
            .fetch_all(self.into_executor())
            .await
        })
    }
    fn get_label_all<'e>(self: Box<Self>) -> BoxFuture<'e, Result<Vec<Label>, sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            sqlx::query_as!(Label, "SELECT * FROM Label")
                .fetch_all(self.into_executor())
                .await
        })
    }
    fn get_label_by_name<'e>(
        self: Box<Self>,
        name: &'e str,
    ) -> BoxFuture<'e, Result<Option<Label>, sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            sqlx::query_as!(Label, "SELECT * FROM Label WHERE name = ?", name)
                .fetch_optional(self.into_executor())
                .await
        })
    }

    fn insert_label<'e>(
        self: Box<Self>,
        label: &'e mut Label,
    ) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            sqlx::query!(
                "INSERT INTO Label (id, name) VALUES (?, ?)",
                label.id,
                label.name
            )
            .execute(self.into_executor())
            .await
            .map(|r| {
                label.id = r.last_insert_rowid();
                r
            })?;
            Ok(())
        })
    }

    fn update_label<'e>(self: Box<Self>, label: &'e Label) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            sqlx::query!(
                r#"UPDATE Label
                SET name = ?
                WHERE id = ?
            "#,
                label.name,
                label.id,
            )
            .execute(self.into_executor())
            .await?;
            Ok(())
        })
    }

    fn delete_label_by_id<'e>(self: Box<Self>, id: i64) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            sqlx::query!("DELETE FROM Label WHERE id = ?", id)
                .execute(self.into_executor())
                .await?;
            Ok(())
        })
    }

    fn insert_labeling<'e>(
        self: Box<Self>,
        label_id: i64,
        image_id: i64,
    ) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            sqlx::query!(
                "INSERT INTO Labeling (label_id, image_id) VALUES (?, ?)",
                label_id,
                image_id
            )
            .execute(self.into_executor())
            .await?;
            Ok(())
        })
    }
}
