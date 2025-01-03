use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use tokio_util::io::ReaderStream;

use crate::{error::Error, state::AppState};

mod refresh;
mod homepage;
mod directory;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(homepage::page))
        .route("/i/:id", get(serve_image))
        .route("/refresh", get(refresh::refresh).post(refresh::refresh))
        .nest("/dir", directory::router())
}

#[axum::debug_handler]
pub async fn serve_image(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let (image, path) = state.get_image_file(id).await?;

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

    let headers = [
        (header::CONTENT_TYPE, content_type.to_owned()),
        (
            header::CONTENT_DISPOSITION,
            format!("inline; filename=\"{}\"", path.display()),
        ),
    ];

    Ok((headers, body))
}
