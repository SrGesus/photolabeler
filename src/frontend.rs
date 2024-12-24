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
    db::{directory::Directory, image::Image},
    AppState,
};

pub async fn serve_images(
    State(AppState(database)): State<AppState>,
    Path((dir, image)): Path<(i64, String)>,
) -> impl IntoResponse {
    tracing::info!("This is crazy we got {dir}/{image}");
    let directory = Directory::get_by_id(&database, dir).await.unwrap();
    let file = fs::File::open(PathBuf::from(directory.path()).join(&image))
        .await
        .unwrap();
    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = Body::from_stream(stream);

    let content_type = match mime_guess::from_path(&image).first_raw() {
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
