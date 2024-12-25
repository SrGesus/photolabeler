use sqlx::Error;

use super::Database;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Label {
    id: i64,
    name: String,
}

impl Label {
    pub fn new(name: String) -> Self {
        Self { id: 0, name }
    }
    pub fn id(&self) -> &i64 {
        &self.id
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub async fn get_by_image(
        Database(pool): &Database,
        image_id: i64,
    ) -> Result<Vec<Label>, Error> {
        sqlx::query_as!(
            Label,
            r#"SELECT id, name
              FROM Label
              INNER JOIN Labeling ON label_id = id
              WHERE image_id = ?
            "#,
            image_id
        )
        .fetch_all(pool)
        .await
    }
}
