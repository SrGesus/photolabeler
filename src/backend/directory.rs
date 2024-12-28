use std::path;

use axum::{
    debug_handler,
    extract::{Form, Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::post,
    Router,
};
use tokio::fs;

use crate::{db::directory::Directory, error::Error, AppState};

use super::add_missing;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct OptionDirectory {
    parent_id: Option<i64>,
    name: Option<String>,
    path: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct InsertDirectory {
    path: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/insert/:dirid", post(insert))
        .route("/insert", post(insert))
}

#[debug_handler]
pub async fn insert(
    State(AppState(database)): State<AppState>,
    dirid: Option<Path<i64>>,
    Form(dir): Form<InsertDirectory>,
) -> Result<impl IntoResponse, Error> {
    let mut new_dir: Directory = if let Some(Path(dirid)) = dirid {
        let parent = Directory::get_by_id(&database, dirid).await?;
        Directory::new(
            Some(parent.id()),
            dir.path.to_string(),
            path::Path::new(parent.path())
                .join(&dir.path)
                .to_string_lossy()
                .into_owned(),
        )
    } else {
        let path = path::Path::new(&dir.path);
        let new_dir = Directory::new(
            None,
            path.file_name().unwrap().to_string_lossy().to_string(),
            path.to_string_lossy().to_string(),
        );
        if fs::metadata(new_dir.path()).await.is_ok() {
            return Err(Error::StatusCode(
                StatusCode::BAD_REQUEST,
                "Path already exists.".to_string(),
            ));
        }
        new_dir
    };

    new_dir.insert(&database).await?;

    fs::create_dir(new_dir.path()).await.ok();

    let new_id = new_dir.id();

    add_missing(&database, new_dir).await?;

    Ok((
        StatusCode::FOUND,
        Redirect::permanent(&format!("/{new_id}")),
    ))
}

pub async fn update(
    State(AppState(database)): State<AppState>,
    Path(id): Path<i64>,
    Form(payload): Form<OptionDirectory>,
) -> Result<StatusCode, Error> {
    let mut dir = Directory::get_by_id(&database, id).await?;

    if let Some(name) = payload.name {
        if name != dir.name() {
            dir.name = name;
        }
    }

    if let Some(parent_id) = payload.parent_id {
        let parent = Directory::get_by_id(&database, parent_id).await?;
        let new_path = path::Path::new(parent.path()).join(dir.name());
        fs::rename(dir.path(), &new_path).await?;
        dir.path = new_path.to_string_lossy().to_string();
    } else if let Some(new_path) = payload.path {
        if dir.parent_id == None && new_path != dir.path() {
            fs::rename(dir.path(), &new_path).await?;
            dir.path = new_path;
        }
    }

    dir.insert(&database).await?;

    Ok(StatusCode::NO_CONTENT)
}
