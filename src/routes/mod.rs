use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use tokio::fs;
use tokio_util::io::ReaderStream;

use crate::{error::Error, state::AppState};

mod directory;
mod home;
mod image;
mod refresh;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/dir", directory::router())
        .nest("/img", image::router())
        .nest("/home", home::router())
        .route("/i/:id", get(serve_image))
        .route("/refresh", get(refresh::refresh).post(refresh::refresh))
        .route("/", get(Redirect::permanent("/home")))
}

#[axum::debug_handler]
pub async fn serve_image(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let path = state.get_image_path(id).await?;
    let image = fs::File::open(&path).await?;

    let body = Body::from_stream(ReaderStream::new(image));

    let content_type = match mime_guess::from_path(&path).first_raw() {
        Some(mime) => mime,
        None => {
            return Err(Error::StatusCode(
                StatusCode::BAD_REQUEST,
                "MIME Type couldn't be determined".to_string(),
            ))
        }
    };

    let mut headers: HeaderMap = HeaderMap::with_capacity(3);

    headers.insert(header::CONTENT_TYPE, content_type.try_into().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("inline; filename=\"{}\"", path.display())
            .try_into()
            .unwrap(),
    );

    if let Ok(modified) = fs::metadata(&path).await?.modified() {
        let modified: DateTime<Utc> = modified.into();
        headers.insert(
            header::LAST_MODIFIED,
            modified
                .format("%a, %d %b %Y %H:%M:%S GMT")
                .to_string()
                .try_into()
                .unwrap(),
        );
    }

    Ok((headers, body))
}
