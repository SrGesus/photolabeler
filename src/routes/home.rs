use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use tera::{Context, Tera};

use crate::{error::Error, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(page).post(register_folder))
        .route("/:id/delete", post(unregister_folder))
        .route("/images", get(all_images))
}

#[axum::debug_handler]
pub async fn all_images(State(state): State<AppState>) -> Result<Html<String>, Error> {
    let directories = state.get_directory_parentless().await?;
    let dir_tree = state.get_dir_tree().await?;
    let images = state.get_image_all().await?;

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
        .register_directory(
            f.path,
            if f.name.is_empty() {
                None
            } else {
                Some(f.name)
            },
        )
        .await?;

    Ok(Redirect::to("/"))
}

pub async fn unregister_folder(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, Error> {
    state.unregister_directory(id).await?;
    Ok(Redirect::to("/"))
}
