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
                r#"SELECT id, name
                FROM Label
                INNER JOIN Labeling ON label_id = id
                WHERE image_id = ?
            "#,
                image_id
            )
            .fetch_all(self.into_executor())
            .await
        })
    }
}
