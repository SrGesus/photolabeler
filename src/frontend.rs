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
    db::{directory::Directory, image::Image, label::Label},
    AppState,
};

pub async fn serve_images(
    State(AppState(database)): State<AppState>,
    Path(image_id): Path<i64>,
) -> impl IntoResponse {

    let image = Image::get_by_id(&database, image_id).await.unwrap().unwrap();
    let directory = Directory::get_by_id(&database, *image.directory_id()).await.unwrap().unwrap();

    let file = fs::File::open(PathBuf::from(directory.path()).join(image.name()))
        .await
        .unwrap();

    let body = Body::from_stream(ReaderStream::new(file));

    let content_type = match mime_guess::from_path(image.name()).first_raw() {
        Some(mime) => mime,
        None => {
            return Err((
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

#[axum::debug_handler]
pub async fn homepage(State(AppState(database)): State<AppState>) -> Html<String> {
    let images = Image::get_all(&database).await.unwrap();

    let tera = Tera::new("templates/**/*.html.tera").unwrap();
    let mut context = Context::new();
    context.insert("images", &images);

    axum::response::Html(tera.render("images.html.tera", &context).unwrap())
}

#[axum::debug_handler]
pub async fn image(State(AppState(database)): State<AppState>, Path(image_id): Path<i64>) -> Html<String> {
    let image = Image::get_by_id(&database, image_id).await.unwrap().unwrap();
    let labels = Label::get_by_image(&database, image_id).await.unwrap();

    let tera = Tera::new("templates/**/*.html.tera").unwrap();
    let mut context = Context::new();
    context.insert("image", &image);
    context.insert("labels", &labels);

    axum::response::Html(tera.render("image.html.tera", &context).unwrap())
}
