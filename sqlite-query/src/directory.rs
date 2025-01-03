use crate::{SqliteApp, SqliteAppExecutor};
use futures::future::BoxFuture;
use query::directory::{AppDirectoryQueryable, Directory};
use sqlx::{Executor, Sqlite};
use std::fmt::Debug;

impl<'k, E> AppDirectoryQueryable<'k> for SqliteApp<E>
where
    SqliteAppExecutor<E>: Executor<'k, Database = Sqlite>,
    E: Debug + Send + Sized + 'k,
{
    fn get_directory_all<'e>(self: Box<Self>) -> BoxFuture<'e, Result<Vec<Directory>, sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(
            sqlx::query_as!(Directory, "SELECT * FROM Directory").fetch_all(self.into_executor()),
        )
    }
    fn get_directory_by_id<'e>(
        self: Box<Self>,
        id: i64,
    ) -> BoxFuture<'e, Result<Directory, sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            sqlx::query_as!(Directory, "SELECT * FROM Directory WHERE id = ?", id)
                .fetch_one(self.into_executor())
                .await
        })
    }
    fn get_directory_by_parent_id<'e>(
        self: Box<Self>,
        par_id: i64,
    ) -> BoxFuture<'e, Result<Vec<Directory>, sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            sqlx::query_as!(
                Directory,
                "SELECT * FROM Directory WHERE parent_id = ?",
                par_id
            )
            .fetch_all(self.into_executor())
            .await
        })
    }
    fn get_directory_parentless<'e>(
        self: Box<Self>,
    ) -> BoxFuture<'e, Result<Vec<Directory>, sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async {
            sqlx::query_as!(Directory, "SELECT * FROM Directory WHERE parent_id IS NULL",)
                .fetch_all(self.into_executor())
                .await
        })
    }

    fn directory_ancestors<'e>(
        self: Box<Self>,
        dir_id: i64,
    ) -> BoxFuture<'e, Result<Vec<Directory>, sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            let mut ancs = sqlx::query_as!(
                Directory,
                r#"WITH RECURSIVE Ancestors AS (
                    SELECT * FROM Directory WHERE id = ?
                    UNION ALL
                    SELECT d.* FROM Directory d
                        INNER JOIN Ancestors a ON d.id = a.parent_id
                ) SELECT * FROM Ancestors;
                "#,
                dir_id,
            )
            .fetch_all(self.into_executor())
            .await?;
            ancs.reverse();
            Ok(ancs)
        })
    }

    fn insert_directory<'e>(
        self: Box<Self>,
        dir: &'e mut Directory,
    ) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            sqlx::query!(
                r#"INSERT INTO Directory (parent_id, name, path)
                        VALUES (?, ?, ?)
                    "#,
                dir.parent_id,
                dir.name,
                dir.path
            )
            .execute(self.into_executor())
            .await
            .map(|r| {
                dir.id = r.last_insert_rowid();
                r
            })?;
            Ok(())
        })
    }

    fn update_directory<'e>(
        self: Box<Self>,
        dir: &'e Directory,
    ) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            sqlx::query!(
                r#"UPDATE Directory
                    SET name = ?, path = ?
                    WHERE id = ?
                "#,
                dir.name,
                dir.path,
                dir.id
            )
            .execute(self.into_executor())
            .await?;
            Ok(())
        })
    }

    fn delete_directory<'e>(self: Box<Self>, id: i64) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e,
    {
        Box::pin(async move {
            sqlx::query!("DELETE FROM Directory WHERE id = ?", id)
                .execute(self.into_executor())
                .await?;
            Ok(())
        })
    }
}
