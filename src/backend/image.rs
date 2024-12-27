use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Form, Router,
};
use tokio::{fs, io::AsyncWriteExt};
use tokio_util::io::ReaderStream;

use crate::{db::image::Image, error::Error, AppState};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct OptionImage {
    directory_id: Option<i64>,
    name: Option<String>,
    notes: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/insert/:dirid",
            post(insert).layer(DefaultBodyLimit::max(8 * 1024 * 1024 * 1024)),
        )
        .route("/update/:id", post(update))
        .route("/delete/:id", post(delete))
        .route("/:id", get(read))
}

pub async fn insert(
    State(AppState(database)): State<AppState>,
    Path(dirid): Path<i64>,
    mut multipart: Multipart,
) -> Result<StatusCode, Error> {
    let mut name = String::new();
    while let Some(field) = multipart.next_field().await? {
        if field.name().is_some_and(|n| n == ("name")) {
            name = field.text().await?;
            continue;
        }

        let original_name = if name.is_empty() {
            field.file_name().ok_or(Error::StatusCode(
                StatusCode::BAD_REQUEST,
                "Image has no file_name".to_owned(),
            ))?
        } else {
            &name
        }
        .to_string();

        let data = field.bytes().await?;

        let (original_left, original_extension) =
            original_name.rsplit_once('.').ok_or(Error::StatusCode(
                StatusCode::BAD_REQUEST,
                format!("Image {original_name} has no file extension"),
            ))?;

        let mut image = Image::new(dirid, original_name.to_string(), String::new());
        let mut tries = 1;
        while let Err(err) = image.insert(&database).await {
            tries += 1;
            image = Image::new(
                dirid,
                format!("{original_left}_{tries}.{original_extension}"),
                String::new(),
            );
            tracing::debug!(
                "Failed inserting image ({err}), trying again with new name={}",
                image.name()
            );
        }

        let path = image.path(&database).await?;
        tracing::info!("Saving file to path: {:?}", &path);
        fs::File::create(path).await?.write_all(&data).await?
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn read(
    State(AppState(database)): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let image = Image::get_by_id(&database, id).await?;

    let file = fs::File::open(image.path(&database).await?).await?;

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

pub async fn update(
    State(AppState(database)): State<AppState>,
    Path(id): Path<i64>,
    Form(payload): Form<OptionImage>,
) -> Result<StatusCode, Error> {
    let mut image = Image::get_by_id(&database, id).await?;
    let original_path = image.path(&database).await?;

    let mut modified = false;

    if let Some(directory_id) = payload.directory_id {
        if directory_id != *image.directory_id() {
            image.directory_id = directory_id;
            modified = true;
        }
    }
    if let Some(name) = payload.name {
        if name != image.name() {
            image.name = name;
            modified = true;
        }
    }

    // If modified move to new path
    if modified {
        fs::rename(original_path, image.path(&database).await?).await?;
    }

    if let Some(notes) = payload.notes {
        if notes != image.notes() {
            image.notes = notes;
            modified = true;
        }
    }

    if modified {
        image.update(&database).await?;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_MODIFIED)
    }
}

pub async fn delete(
    State(AppState(database)): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, Error> {
    let image = Image::get_by_id(&database, id).await?;
    fs::remove_file(image.path(&database).await?).await?;
    image.delete(&database).await?;
    Ok(StatusCode::NO_CONTENT)
}
