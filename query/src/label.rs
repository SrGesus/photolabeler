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
    fn get_label_all<'e>(self: Box<Self>) -> BoxFuture<'e, Result<Vec<Label>, sqlx::Error>>
    where
        'k: 'e;
    fn get_label_by_image_id<'e>(
        self: Box<Self>,
        image_id: i64,
    ) -> BoxFuture<'e, Result<Vec<Label>, sqlx::Error>>
    where
        'k: 'e;
    fn get_label_by_name<'e>(
        self: Box<Self>,
        name: &'e str,
    ) -> BoxFuture<'e, Result<Option<Label>, sqlx::Error>>
    where
        'k: 'e;

    fn insert_label<'e>(
        self: Box<Self>,
        label: &'e mut Label,
    ) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e;

    fn update_label<'e>(
        self: Box<Self>,
        label: &'e Label,
    ) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e;

    fn delete_label_by_id<'e>(
        self: Box<Self>,
        label_id: i64,
    ) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e;

    fn insert_labeling<'e>(
        self: Box<Self>,
        label_id: i64,
        image_id: i64,
    ) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e;

    fn delete_labeling<'e>(
        self: Box<Self>,
        label_id: i64,
        image_id: i64,
    ) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e;
}
