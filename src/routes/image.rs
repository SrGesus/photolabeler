use axum::{
    extract::{Path, State},
    response::Html,
};
use query::label::Label;
use tera::{Context, Tera};

use crate::{error::Error, state::AppState};

#[axum::debug_handler]
pub async fn page(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, Error> {
    let image = state.get_image_by_id(id).await?;
    // let labels = state.get_label ....
    let labels: Vec<Label> = vec![];
    let parents = state.directory_ancestors(image.directory_id).await?;

    let tera = Tera::new("templates/**/*").unwrap();
    let mut context = Context::new();
    context.insert("image", &image);
    context.insert("labels", &labels);
    context.insert("dir_trail", &parents);

    Ok(Html(tera.render("image.tera", &context).unwrap()))
}
