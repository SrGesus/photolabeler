use axum::{extract::State, http::StatusCode, routing::post, Router};
use futures::{stream::FuturesUnordered, StreamExt};
use std::path;
use tokio::fs::{self, DirEntry};

use crate::{
    db::{directory::Directory, image::Image, Database},
    error::Error,
    AppState,
};

pub(crate) mod directory;
pub(crate) mod image;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/refresh", post(refresh).get(refresh))
        // .route("/images", get(get_all_images))
        .nest("/img", image::router())
        .nest("/dir", directory::router())
}

// #[axum::debug_handler]
// pub async fn post_directory(
//     State(AppState(database)): State<AppState>,
//     Path(parent_dir_id): Path<i64>,
//     mut multipart: Multipart,
// ) -> impl IntoResponse {
//     // let mut name = String::new();
//     if let Some(field) = multipart.next_field().await.unwrap() {
//         let name = field.text().await.unwrap();

//         let parent_dir = Directory::get_by_id(&database, parent_dir_id)
//             .await
//             .unwrap();

//         let path = path::PathBuf::from(parent_dir.unwrap().path()).join(&name);

//         fs::create_dir(&path).await.unwrap();

//         Directory::new(
//             Some(parent_dir_id),
//             name,
//             path.to_string_lossy().into_owned(),
//         )
//         .insert(&database)
//         .await
//         .unwrap();
//     }
//     axum::http::StatusCode::NO_CONTENT
// }

pub async fn check_image(
    database: &Database,
    path: &path::Path,
    image: Image,
) -> Result<(), Error> {
    if fs::metadata(path.join(image.name())).await.is_err() {
        image.delete(&database).await?;
    }
    Ok(())
}

pub async fn check_directory(database: &Database, dir: Directory) -> Result<(), Error> {
    if fs::metadata(dir.path()).await.is_err() {
        dir.delete(&database).await?;
    }
    Ok(())
}

pub async fn delete_missing(database: &Database, dir: &Directory) -> Result<(), Error> {
    // Directories in directory
    let directories = Directory::get_by_parent_id(&database, dir.id()).await?;
    let dir_path = path::Path::new(dir.path());

    let mut dir_futures = directories
        .into_iter()
        .map(|dir| check_directory(database, dir))
        .collect::<FuturesUnordered<_>>();
    while let Some(res) = dir_futures.next().await {
        res?;
    }

    // Images in directory
    let images = Image::get_by_directory(&database, dir.id()).await?;

    let mut im_futures = images
        .into_iter()
        .map(|im| check_image(database, dir_path, im))
        .collect::<FuturesUnordered<_>>();
    while let Some(res) = im_futures.next().await {
        res?;
    }
    Ok(())
}

pub async fn add_missing_file(
    database: &Database,
    dir: &Directory,
    file: DirEntry,
) -> Result<Option<Directory>, Error> {
    let mut file_type = file.file_type().await?;
    let mut file_path = file.path();
    let name = file.file_name().into_string().unwrap();

    // Follow symlinks
    while file_type.is_symlink() {
        file_path = fs::read_link(file_path).await?;
        file_type = fs::metadata(&file_path).await?.file_type();
    }

    if file_type.is_dir() {
        let mut new_dir = Directory::new(
            Some(dir.id()),
            name,
            file.path().into_os_string().into_string().unwrap(),
        );
        if let Ok(_) = new_dir.insert(&database).await {
            return Ok(Some(new_dir));
        }
    } else if file_type.is_file()
        && mime_guess::from_path(file.path())
            .first_raw()
            .is_some_and(|content| content.contains("image"))
    {
        let mut i = Image::new(dir.id(), name, String::new());
        // Ignore errors on insert if image is already present on the database
        i.insert(&database).await.ok();
    }
    Ok(None)
}

pub async fn add_missing(database: &Database, directory: Directory) -> Result<(), Error> {
    let mut v = vec![directory];
    while let Some(dir) = v.pop() {
        if fs::metadata(dir.path()).await.is_err() {
            continue;
        }
        // Insert files from directory into database
        let mut entries = fs::read_dir(dir.path()).await?;
        let mut files = Vec::new();
        while let Some(file) = entries.next_entry().await.unwrap() {
            files.push(file);
        }

        let mut futures = files
            .into_iter()
            .map(|file| add_missing_file(database, &dir, file))
            .collect::<FuturesUnordered<_>>();

        while let Some(res) = futures.next().await {
            if let Ok(Some(new_dir)) = res {
                v.push(new_dir);
            }
        }
    }
    Ok(())
}

pub async fn refresh(State(AppState(database)): State<AppState>) -> Result<StatusCode, Error> {
    let directories = Directory::get_all(&database).await.unwrap();
    for dir in directories {
        // Delete image from database if file is gone
        delete_missing(&database, &dir).await?;
        add_missing(&database, dir).await?;
    }
    Ok(StatusCode::NO_CONTENT)
}
