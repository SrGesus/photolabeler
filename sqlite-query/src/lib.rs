use async_trait::async_trait;
use query::{
    directory::{AppPoolDirectory, Directory},
    image::{AppPoolImage, Image},
    label::{AppPoolLabel, Label},
    AppPool,
};
use sqlx::{migrate, Pool, Sqlite, SqlitePool};

pub struct SqliteAppPool(Pool<Sqlite>);

impl SqliteAppPool {
    pub async fn new(url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(url).await?;

        // Run migrations that on compile-time are in '../migrations'
        migrate!("../migrations").run(&pool).await?;

        Ok(Self(pool))
    }
}

impl AppPool for SqliteAppPool {}

#[async_trait]
impl AppPoolLabel for SqliteAppPool {
    async fn get_label_by_image_id(&self, image_id: i64) -> Result<Vec<Label>, sqlx::Error> {
        sqlx::query_as!(
            Label,
            r#"SELECT id, name
                FROM Label
                INNER JOIN Labeling ON label_id = id
                WHERE image_id = ?
            "#,
            image_id
        )
        .fetch_all(&self.0)
        .await
    }
}

#[async_trait]
impl AppPoolDirectory for SqliteAppPool {
    async fn get_directory_all(&self) -> Result<Vec<Directory>, sqlx::Error> {
        Ok(sqlx::query_as!(Directory, "SELECT * FROM Directory")
            .fetch_all(&self.0)
            .await?)
    }
    async fn get_directory_by_id(&self, id: i64) -> Result<Directory, sqlx::Error> {
        Ok(
            sqlx::query_as!(Directory, "SELECT * FROM Directory WHERE id = ?", id)
                .fetch_one(&self.0)
                .await?,
        )
    }
    async fn get_directory_by_parent_id(&self, par_id: i64) -> Result<Vec<Directory>, sqlx::Error> {
        Ok(sqlx::query_as!(
            Directory,
            "SELECT * FROM Directory WHERE parent_id = ?",
            par_id
        )
        .fetch_all(&self.0)
        .await?)
    }
    async fn get_directory_parentless(&self) -> Result<Vec<Directory>, sqlx::Error> {
        Ok(
            sqlx::query_as!(Directory, "SELECT * FROM Directory WHERE parent_id IS NULL",)
                .fetch_all(&self.0)
                .await?,
        )
    }

    async fn directory_ancestors(&self, dir: &Directory) -> Result<Vec<Directory>, sqlx::Error> {
        let mut parent_id = dir.parent_id;
        let mut parent_directories = Vec::new();
        while let Some(id) = parent_id {
            let par_dir = self.get_directory_by_id(id).await?;
            parent_id = par_dir.parent_id;
            parent_directories.push(par_dir);
        }
        parent_directories.reverse();
        Ok(parent_directories)
    }

    async fn insert_directory(&self, dir: &mut Directory) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO Directory (parent_id, name, path)
                VALUES (?, ?, ?)
            "#,
            dir.parent_id,
            dir.name,
            dir.path
        )
        .execute(&self.0)
        .await
        .map(|r| {
            dir.id = r.last_insert_rowid();
            r
        })?;
        Ok(())
    }

    async fn update_directory(&self, dir: &Directory) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE Directory
                SET name = ?, path = ?
                WHERE id = ?
            "#,
            dir.name,
            dir.path,
            dir.id
        )
        .execute(&self.0)
        .await?;
        Ok(())
    }

    async fn delete_directory(&self, dir_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM Directory WHERE id = ?", dir_id)
            .execute(&self.0)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl AppPoolImage for SqliteAppPool {
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
    async fn get_image_by_directory_id(&self, dir_id: i64) -> Result<Vec<Image>, sqlx::Error> {
        Ok(
            sqlx::query_as!(Image, "SELECT * FROM Image WHERE directory_id = ?", dir_id)
                .fetch_all(&self.0)
                .await?,
        )
    }
    async fn get_image_by_label_id(&self, lab_id: i64) -> Result<Vec<Image>, sqlx::Error> {
        Ok(sqlx::query_as!(
            Image,
            r#"SELECT id, directory_id, name, notes
                    FROM Image
                    INNER JOIN Labeling ON image_id = id
                    WHERE label_id = ?"#,
            lab_id
        )
        .fetch_all(&self.0)
        .await?)
    }

    async fn insert_image(&self, image: &mut Image) -> Result<(), sqlx::Error> {
        sqlx::query!(
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
        })?;
        Ok(())
    }

    async fn update_image(&self, image: &Image) -> Result<(), sqlx::Error> {
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
        .execute(&self.0)
        .await?;
        Ok(())
    }
    async fn update_image_directory_many(
        &self,
        ids: Vec<i64>,
        dir_id: i64,
    ) -> Result<(), sqlx::Error> {
        let json_ids = sqlx::types::Json(ids);
        sqlx::query!(
            r#"UPDATE Image
                SET directory_id = ?
                WHERE id IN (SELECT value from json_each(?))
            "#,
            dir_id,
            json_ids
        )
        .execute(&self.0)
        .await?;
        Ok(())
    }

    async fn delete_image_by_id(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM Image WHERE id = ?", id)
            .execute(&self.0)
            .await?;
        Ok(())
    }
    async fn delete_image_by_id_many(&self, ids: Vec<i64>) -> Result<(), sqlx::Error> {
        let json_ids = sqlx::types::Json(ids);
        sqlx::query!(
            "DELETE FROM Image WHERE id IN (SELECT value from json_each(?))",
            json_ids
        )
        .execute(&self.0)
        .await?;
        Ok(())
    }
}
