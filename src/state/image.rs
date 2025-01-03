use std::path::Path;

use axum::http::StatusCode;
use futures::{stream::FuturesUnordered, StreamExt};
use query::image::Image;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tokio_util::bytes::Bytes;

use crate::error::Error;

use super::AppState;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UpdateImage {
    directory_id: Option<i64>,
    name: Option<String>,
    notes: Option<String>,
}

impl AppState {
    pub async fn get_image_file(&self, id: i64) -> Result<File, Error> {
        let image = self.pool.queryable().get_image_by_id(id).await?;

        let path = self.pool.queryable().get_image_path(&image).await?;

        Ok(fs::File::open(path).await?)
    }

    pub async fn get_image_all(&self) -> Result<Vec<Image>, sqlx::Error> {
        Ok(self.pool.queryable().get_image_all().await?)
    }

    pub async fn get_image_by_id(&self, id: i64) -> Result<Image, Error> {
        Ok(self.pool.queryable().get_image_by_id(id).await?)
    }

    pub async fn get_image_by_directory_id(&self, dir_id: i64) -> Result<Vec<Image>, Error> {
        Ok(self.pool.queryable().get_image_by_directory_id(dir_id).await?)
    }

    pub async fn get_image_by_label_id(&self, lab_id: i64) -> Result<Vec<Image>, Error> {
        Ok(self.pool.queryable().get_image_by_label_id(lab_id).await?)
    }

    pub async fn save_image(&self, mut image: Image, image_bytes: Bytes) -> Result<(), Error> {
        let original_name = image.name.clone();

        let (original_left, original_extension) =
            original_name.rsplit_once('.').ok_or(Error::StatusCode(
                StatusCode::BAD_REQUEST,
                format!("Image {original_name} has no file extension"),
            ))?;

        if let None = mime_guess::from_ext(original_extension).first_raw() {
            return Err(Error::StatusCode(
                StatusCode::BAD_REQUEST,
                format!("MIME Type couldn't be determined for {original_name}"),
            ));
        }

        let mut tries = 1;

        let mut transaction = self.pool.transaction().await?;
        match {
            while let Ok(_) = transaction
                .queryable()
                .get_image_by_name_in_dir(image.directory_id, &image.name)
                .await
            {
                tries += 1;
                image.name = format!("{original_left}_{tries}.{original_extension}");
            }

            transaction.queryable().insert_image(&mut image).await?;

            let path = transaction.queryable().get_image_path(&image).await?;
            tracing::info!("Saving file to path: {:?}", &path);
            fs::File::create(path).await?.write_all(&image_bytes).await
        } {
            Err(err) => {
                transaction.rollback();
                Err(err)?
            }
            Ok(_) => {
                transaction.commit();
                Ok(())
            }
        }
    }

    pub async fn update_image(&self, id: i64, update: UpdateImage) -> Result<(), Error> {
        // Get old image
        let mut image = self.pool.queryable().get_image_by_id(id).await?;
        // let (old_dir, old_name) = (image.directory_id, image.name);

        if update.directory_id.is_some_and(|d| d != image.directory_id)
            || update.name.as_ref().is_some_and(|n| n != &image.name)
        {
            let old_path = self.pool.queryable().get_image_path(&image).await?;
            image.directory_id = update.directory_id.unwrap_or(image.directory_id);
            image.name = update.name.unwrap_or(image.name);
            let new_path = self.pool.queryable().get_image_path(&image).await?;

            Self::move_file(old_path, new_path).await?;
        }

        image.notes = update.notes.unwrap_or(image.notes);

        self.pool.queryable().update_image(&image);

        Ok(())
    }

    async fn move_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> tokio::io::Result<()> {
        // Attempt to rename file, if not possible try copy
        if let Err(_) = fs::rename(&from, &to).await {
            fs::copy(&from, to).await?;
            fs::remove_file(from).await?;
        }
        Ok(())
    }

    async fn move_image_to_new_dir(&self, image_id: i64, new_path: &Path) -> Result<(), Error> {
        // Since this query is not made from the transaction, the directory should still
        // be the old one because it should not read uncommitted changes
        let image = self.pool.queryable().get_image_by_id(image_id).await?;
        let old_path = self.pool.queryable().get_image_path(&image).await?;
        Ok(Self::move_file(old_path, new_path).await?)
    }

    async fn remove_image(&self, image_id: i64) -> Result<(), Error> {
        let image = self.pool.queryable().get_image_by_id(image_id).await?;
        let old_path = self.pool.queryable().get_image_path(&image).await?;
        Ok(fs::remove_file(old_path).await?)
    }

    pub async fn move_images(&self, ids: Vec<i64>, new_dir_id: i64) -> Result<(), Error> {
        let mut transaction = self.pool.transaction().await?;

        match {
            let new_dir = transaction
                .queryable()
                .get_directory_by_id(new_dir_id)
                .await?;
            let new_path = Path::new(&new_dir.path);

            transaction
                .queryable()
                .update_image_directory_many(&ids, new_dir_id)
                .await?;

            let mut move_futures = ids
                .iter()
                .map(|id| self.move_image_to_new_dir(*id, new_path))
                .collect::<FuturesUnordered<_>>();

            // Consider making this reversible instead of blowing everything up
            // But this should only fail if there isn't enough space on disk
            while let Some(res) = move_futures.next().await {
                res?;
            }

            Ok(())
        } {
            Err(err) => {
                tracing::error!("Failed to move images: {err}");
                transaction.rollback().await?;
                Err(err)
            }
            Ok(_) => {
                transaction.commit().await?;
                Ok(())
            }
        }
    }

    pub async fn delete_image(&self, id: i64) -> Result<(), Error> {
        let image = self.pool.queryable().get_image_by_id(id).await?;
        fs::remove_file(self.pool.queryable().get_image_path(&image).await?).await?;
        Ok(self.pool.queryable().delete_image_by_id(id).await?)
    }

    pub async fn delete_image_many(&self, ids: &Vec<i64>) -> Result<(), Error> {
        let mut transaction = self.pool.transaction().await?;

        match {
            transaction.queryable().delete_image_by_id_many(ids).await?;

            let mut remove_futures = ids
                .iter()
                .map(|id| self.remove_image(*id))
                .collect::<FuturesUnordered<_>>();

            while let Some(res) = remove_futures.next().await {
                res.ok();
            }
            Ok(())
        } {
            Err(err) => {
                tracing::error!("Failed to delete images: {err}");
                transaction.rollback().await?;
                Err(err)
            }
            Ok(_) => {
                transaction.commit().await?;
                Ok(())
            }
        }
    }
}