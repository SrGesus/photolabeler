use futures::future::BoxFuture;

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

pub trait AppLabelQueryable<'k> {
    fn get_label_by_image_id<'e>(
        self: Box<Self>,
        image_id: i64,
    ) -> BoxFuture<'e, Result<Vec<Label>, sqlx::Error>>
    where
        'k: 'e;
}
