use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use tera::{Context, Tera};

use crate::{error::Error, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(page))
        .route("/dir", post(register_folder))
        .route("/dir/:id/delete", post(unregister_folder))
        .route("/images", get(all_images))
}

#[derive(Debug, serde::Deserialize)]
pub struct ImageFilter {
    labels: Option<String>,
}

#[axum::debug_handler]
pub async fn all_images(
    State(state): State<AppState>,
    Query(filter): Query<ImageFilter>,
) -> Result<Html<String>, Error> {
    let directories = state.get_directory_parentless().await?;
    let dir_tree = state.get_dir_tree().await?;
    let labels = filter
        .labels
        .map(|l| l.split(',').map(|s| s.to_string()).collect());
    let images = state.get_image_all(labels).await?;

    let tera = Tera::new("templates/**/*").unwrap();
    let mut context = Context::new();
    context.insert("images", &images);
    context.insert("directories", &directories);
    context.insert("dir_tree", &dir_tree);

    Ok(Html(tera.render("image-list.tera", &context).unwrap()))
}

pub async fn page(State(state): State<AppState>) -> Result<Html<String>, Error> {
    let directories = state.get_directory_parentless().await?;

    let tera = Tera::new("templates/**/*").unwrap();
    let mut context = Context::new();
    context.insert("directories", &directories);

    Ok(Html(tera.render("homepage.tera", &context).unwrap()))
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct NewFolder {
    path: String,
    name: String,
}

pub async fn register_folder(
    State(state): State<AppState>,
    Form(f): Form<NewFolder>,
) -> Result<impl IntoResponse, Error> {
    state
        .register_directory(f.path, Some(f.name).filter(|v| !v.is_empty()))
        .await.map_err(|err| {
            tracing::error!("{}", err);
            err
        })?;

    Ok(Redirect::to("/home"))
}

pub async fn unregister_folder(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, Error> {
    state.unregister_directory(id).await?;
    Ok(Redirect::to("/home"))
}
