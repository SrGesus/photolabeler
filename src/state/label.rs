use query::label::Label;

use crate::error::Error;

use super::AppState;

impl AppState {
    pub async fn get_label_by_image_id(&self, image_id: i64) -> Result<Vec<Label>, Error> {
        Ok(self
            .pool
            .queryable()
            .get_label_by_image_id(image_id)
            .await?)
    }
    pub async fn get_label_by_name(&self, name: &str) -> Result<Label, Error> {
        Ok(match self.pool.queryable().get_label_by_name(name).await? {
            Some(label) => label,
            None => {
                let mut label = Label::new(name.to_owned());
                self.insert_label(&mut label).await?;
                label
            }
        })
    }
    pub async fn get_label_all(&self) -> Result<Vec<Label>, Error> {
        Ok(self.pool.queryable().get_label_all().await?)
    }
    pub async fn insert_label(&self, label: &mut Label) -> Result<(), Error> {
        Ok(self.pool.queryable().insert_label(label).await?)
    }
    pub async fn update_label(&self, label: &Label) -> Result<(), Error> {
        Ok(self.pool.queryable().update_label(label).await?)
    }
    pub async fn delete_label_by_id(&self, id: i64) -> Result<(), Error> {
        Ok(self.pool.queryable().delete_label_by_id(id).await?)
    }
    pub async fn insert_labeling(&self, label_id: i64, image_id: i64) -> Result<(), sqlx::Error> {
        Ok(self
            .pool
            .queryable()
            .insert_labeling(label_id, image_id)
            .await?)
    }
    pub async fn delete_labeling(&self, label_id: i64, image_id: i64) -> Result<(), sqlx::Error> {
        Ok(self
            .pool
            .queryable()
            .delete_labeling(label_id, image_id)
            .await?)
    }
}
