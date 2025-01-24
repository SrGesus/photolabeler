use axum::{extract::State, response::{Html, IntoResponse, Redirect}, Form};
use tera::{Context, Tera};

use crate::{error::Error, state::AppState};

#[axum::debug_handler]
pub async fn page(State(state): State<AppState>) -> Result<Html<String>, Error> {
    let directories = state.get_directory_parentless().await?;

    let tera = Tera::new("templates/**/*").unwrap();
    let mut context = Context::new();
    context.insert("directories", &directories);

    Ok(Html(tera.render("homepage.tera", &context).unwrap()))
}
