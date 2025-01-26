use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
    routing::{get, post},
    Form, Router,
};
use tera::{Context, Tera};

use crate::{error::Error, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:id", get(page))
        .route("/:id/label", post(add_label))
        .route("/:id/label/:label_id/delete", get(delete_label))
}

#[axum::debug_handler]
pub async fn page(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, Error> {
    let image = state.get_image_by_id(id).await?;
    // let labels = state.get_label ....
    let labels = state.get_label_by_image_id(id).await?;
    let parents = state.directory_ancestors(image.directory_id).await?;

    let tera = Tera::new("templates/**/*").unwrap();
    let mut context = Context::new();
    context.insert("image", &image);
    context.insert("labels", &labels);
    context.insert("dir_trail", &parents);

    Ok(Html(tera.render("image.tera", &context).unwrap()))
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AddLabel {
    name: String,
}

#[axum::debug_handler]
pub async fn delete_label(
    State(state): State<AppState>,
    Path((image_id, label_id)): Path<(i64, i64)>,
) -> Result<Redirect, Error> {
    state.delete_labeling(label_id, image_id).await?;

    Ok(Redirect::to(&format!("/img/{image_id}")))
}

pub async fn add_label(
    State(state): State<AppState>,
    Path(image_id): Path<i64>,
    Form(add_label): Form<AddLabel>,
) -> Result<Redirect, Error> {
    let label = state.get_label_by_name(&add_label.name).await?;

    state.insert_labeling(label.id, image_id).await?;

    Ok(Redirect::to(&format!("/img/{image_id}")))
}
