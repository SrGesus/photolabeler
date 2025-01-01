use async_trait::async_trait;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Label {
    pub id: i64,
    pub name: String,
}

impl Label {
    pub fn new(name: String) -> Self {
        Self { id: 0, name }
    }
}

#[async_trait]
pub trait AppPoolLabel {
    async fn get_label_by_image_id(&self, image_id: i64) -> Result<Vec<Label>, sqlx::Error>;
}
