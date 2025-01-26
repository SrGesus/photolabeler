use futures::future::BoxFuture;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Directory {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub path: String,
}

impl Directory {
    pub fn new(parent_id: Option<i64>, name: String, path: String) -> Self {
        Self {
            id: 0,
            parent_id,
            name,
            path,
        }
    }
}

pub trait AppDirectoryQueryable<'k> {
    fn get_directory_all<'e>(self: Box<Self>) -> BoxFuture<'e, Result<Vec<Directory>, sqlx::Error>>
    where
        'k: 'e;
    fn get_directory_by_id<'e>(
        self: Box<Self>,
        id: i64,
    ) -> BoxFuture<'e, Result<Directory, sqlx::Error>>
    where
        'k: 'e;
    fn get_directory_by_parent_id<'e>(
        self: Box<Self>,
        par_id: i64,
    ) -> BoxFuture<'e, Result<Vec<Directory>, sqlx::Error>>
    where
        'k: 'e;
    fn get_directory_parentless<'e>(
        self: Box<Self>,
    ) -> BoxFuture<'e, Result<Vec<Directory>, sqlx::Error>>
    where
        'k: 'e;

    fn directory_ancestors<'e>(
        self: Box<Self>,
        dir_id: i64,
    ) -> BoxFuture<'e, Result<Vec<Directory>, sqlx::Error>>
    where
        'k: 'e;

    fn insert_directory<'e>(
        self: Box<Self>,
        dir: &'e mut Directory,
    ) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e;

    fn update_directory<'e>(
        self: Box<Self>,
        dir: &'e Directory,
    ) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e;

    fn delete_directory<'e>(self: Box<Self>, id: i64) -> BoxFuture<'e, Result<(), sqlx::Error>>
    where
        'k: 'e;
}
