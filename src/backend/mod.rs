use axum::{
    extract::{Multipart, Path, State},
    routing::{get, post},
    Json, Router,
};
use std::{path, sync::Arc};
use tokio::{fs, io::AsyncWriteExt};

use crate::{
    db::{directory::Directory, image::Image, Database},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/refresh", post(refresh).get(refresh))
        .route("/images", get(get_all_images))
        .route("/dir/:dir", post(post_image))
}

#[axum::debug_handler]
pub async fn get_all_images(State(AppState(database)): State<AppState>) -> Json<Vec<Image>> {
    let images = Image::get_all(&database).await.unwrap();
    dbg!(&images);
    Json(images)
}

#[axum::debug_handler]
pub async fn post_image(
    State(AppState(database)): State<AppState>,
    Path(dir): Path<i64>,
    mut multipart: Multipart,
) {
    let dir = Directory::get_by_id(&database, dir).await.unwrap().unwrap();
    delete_missing(&database, &dir).await;
    while let Some(field) = multipart.next_field().await.unwrap() {
        let original_name = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        println!("Length of `{}` is {} bytes", original_name, data.len());
        let (original_left, original_extension) = original_name.rsplit_once('.').unwrap();

        let mut name = original_name.to_owned();
        let mut tries = 1;
        while let Err(_) = Image::new(*dir.id(), name.to_owned(), None)
            .insert(&database)
            .await
        {
            tries += 1;
            name = format!("{original_left}_{tries}.{original_extension}");
            tracing::info!("Failed, trying again with new name={}", &name);
        }

        let path = path::Path::new(dir.path()).join(&name);
        tracing::info!("Saving file to path: {:?}", &path);
        fs::File::create(path).await.unwrap().write_all(&data).await.unwrap();
    }
}

pub async fn delete_missing(database: &Database, dir: &Directory) {
    let dir_path = path::Path::new(dir.path());
    let images = Image::get_by_directory(&database, *dir.id()).await.unwrap();
    for im in images {
        if !fs::metadata(dir_path.join(im.name())).await.is_ok() {
            im.delete(&database).await.unwrap();
        }
    }
}
pub async fn add_missing(database: &Database, dir: &Directory) {
    // Insert files from directory into database
    let mut entries = fs::read_dir(dir.path()).await.unwrap();
    while let Some(file) = entries.next_entry().await.unwrap() {
        let name = file.file_name();
        let i = Image::new(*dir.id(), name.into_string().unwrap(), None);
        // Ignore errors on insert if image is already present on the database
        i.insert(&database).await.ok();
    }
}

pub async fn refresh(State(AppState(database)): State<AppState>) {
    let directories = Directory::get_all(&database).await.unwrap();
    for dir in directories {
        // Delete image from database if file is gone
        delete_missing(&database, &dir).await;
        add_missing(&database, &dir).await;
    }
}
