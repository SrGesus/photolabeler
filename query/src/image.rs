use futures::future::BoxFuture;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Image {
    pub id: i64,
    pub directory_id: i64,
    pub name: String,
    pub notes: String,
}

impl Image {
    pub fn new(directory_id: i64, name: String, notes: String) -> Self {
        Self {
            id: 0,
            directory_id,
            name,
            notes,
        }
    }
}

pub trait AppImageQueryable<'k> {
    fn get_image_all<'e>(self: Box<Self>) -> BoxFuture<'e, Result<Vec<Image>, sqlx::Error>>
    where
        'k: 'e;
    fn get_image_by_id<'e>(self: Box<Self>, id: i64) -> BoxFuture<'e, Result<Image, sqlx::Error>>
    where
        'k: 'e;
    fn get_image_by_name_in_dir<'e>(
        self: Box<Self>,
        dir_id: i64,
        name: &'e str,
    ) -> BoxFuture<'e, Result<Image, sqlx::Error>>
    where
        'k: 'e;
    fn get_image_by_directory_id<'e>(
        self: Box<Self>,
        dir_id: i64,
    ) -> BoxFuture<'e, Result<Vec<Image>, sqlx::Error>>
    where
        'k: 'e;
    fn get_image_by_label_id<'e>(
        self: Box<Self>,
        lab_id: i64,
    ) -> BoxFuture<'e, Result<Vec<Image>, sqlx::Error>>
    where
        'k: 'e;
    fn insert_image<'e>(
        self: Box<Self>,
        image: &'e mut Image,
    ) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e;
    fn update_image<'e>(
        self: Box<Self>,
        image: &'e Image,
    ) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e;
    fn update_image_directory_many<'e>(
        self: Box<Self>,
        ids: &'e Vec<i64>,
        dir_id: i64,
    ) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e;
    fn delete_image_by_id<'e>(self: Box<Self>, id: i64) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e;
    fn delete_image_by_id_many<'e>(
        self: Box<Self>,
        ids: &'e Vec<i64>,
    ) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e;
}
