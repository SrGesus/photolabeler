use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
};
use std::path::PathBuf;
use tera::{Context, Tera};
use tokio::fs;
use tokio_util::io::ReaderStream;

use crate::{
    db::{
        directory::{self, Directory},
        image::Image,
        label::Label,
    },
    error::Error,
    AppState,
};

pub async fn serve_images(
    State(AppState(database)): State<AppState>,
    Path(image_id): Path<i64>,
) -> impl IntoResponse {
    let image = Image::get_by_id(&database, image_id).await?;
    let directory = Directory::get_by_id(&database, *image.directory_id()).await?;

    let file = fs::File::open(PathBuf::from(directory.path()).join(image.name())).await?;

    let body = Body::from_stream(ReaderStream::new(file));

    let content_type = match mime_guess::from_path(image.name()).first_raw() {
        Some(mime) => mime,
        None => {
            return Err(Error::StatusCode(
                StatusCode::BAD_REQUEST,
                "MIME Type couldn't be determined".to_string(),
            ))
        }
    };

    let headers = [
        (header::CONTENT_TYPE, content_type.to_owned()),
        (
            header::CONTENT_DISPOSITION,
            format!("inline; filename=\"{:?}\"", image),
        ),
    ];

    Ok((headers, body))
}

// #[axum::debug_handler]
// pub async fn homepage(State(AppState(database)): State<AppState>) -> Html<String> {
//     let images = Image::get_all(&database).await.unwrap();
//     let directories = Directory::get_by_parentless(&database).await.unwrap();

//     let tera = Tera::new("templates/**/*.html.tera").unwrap();
//     let mut context = Context::new();
//     context.insert("images", &images);
//     context.insert("directories", &directories);

//     axum::response::Html(tera.render("homepage.html.tera", &context).unwrap())
// }
#[axum::debug_handler]
pub async fn homepage(State(AppState(database)): State<AppState>) -> Result<Html<String>, Error> {
    let images = Image::get_all(&database).await?;
    let directories = Directory::get_by_parentless(&database).await?;
    let parents: Vec<Directory> = Vec::new();

    let tera = Tera::new("templates/**/*.html.tera").unwrap();
    let mut context = Context::new();
    context.insert("images", &images);
    context.insert("directories", &directories);
    context.insert("dir_parents", &parents);
    context.insert("dir_id", &0);

    Ok(axum::response::Html(
        tera.render("images.html.tera", &context).unwrap(),
    ))
}

#[axum::debug_handler]
pub async fn images(
    State(AppState(database)): State<AppState>,
    Path(directory_id): Path<i64>,
) -> Result<Html<String>, Error> {
    tracing::info!("Begin generating for {}", directory_id);
    let images = Image::get_by_directory(&database, directory_id).await?;
    let directories = Directory::get_by_parent_id(&database, directory_id).await?;
    let dir = Directory::get_by_id(&database, directory_id).await?;
    let parents = dir.parent_directories(&database).await?;

    let tera = Tera::new("templates/**/*.html.tera").unwrap();
    let mut context = Context::new();
    context.insert("images", &images);
    context.insert("directories", &directories);
    context.insert("dir_parents", &parents);
    context.insert("dir_id", &directory_id);

    let res = axum::response::Html(tera.render("images.html.tera", &context).unwrap());
    tracing::info!("Finish generating for {}", directory_id);
    Ok(res)
}

#[axum::debug_handler]
pub async fn image(
    State(AppState(database)): State<AppState>,
    Path(image_id): Path<i64>,
) -> Result<Html<String>, Error> {
    let image = Image::get_by_id(&database, image_id).await?;
    let labels = Label::get_by_image(&database, image_id).await?;
    let parents = Directory::get_by_id(&database, *image.directory_id())
        .await?
        .parent_directories(&database)
        .await?;

    let tera = Tera::new("templates/**/*.html.tera").unwrap();
    let mut context = Context::new();
    context.insert("image", &image);
    context.insert("labels", &labels);
    context.insert("dir_parents", &parents);
    Ok(axum::response::Html(
        tera.render("image.html.tera", &context).unwrap(),
    ))
}
