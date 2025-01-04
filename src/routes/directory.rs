use std::{collections::BTreeMap, fmt::format};

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Form, Json, Router,
};
use query::image::Image;
use tera::{Context, Tera};

use crate::{
    error::Error,
    state::{directory::DirTree, AppState},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:id", get(page))
        .route("/:id/img", post(upload_image))
        .route("/:id/dir", post(create_directory))
        .route("/:id/img/delete", post(delete_images))
        .route("/:id/img/move", post(move_images))
        .route("/tree", get(tree))
}

#[axum::debug_handler]
pub async fn tree(State(state): State<AppState>) -> Result<Json<Vec<DirTree>>, Error> {
    Ok(Json(state.get_dir_tree().await?))
}

#[axum::debug_handler]
pub async fn page(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, Error> {
    let directories: Vec<query::directory::Directory> =
        state.get_directory_by_parent_id(id).await?;
    let ancestors = state.directory_ancestors(id).await?;
    let images = state.get_image_by_directory_id(id).await?;
    let dir_tree = state.get_dir_tree().await?;

    let tera = Tera::new("templates/**/*").unwrap();
    let mut context = Context::new();
    context.insert("directories", &directories);
    context.insert("images", &images);
    context.insert("dir", &ancestors.last().unwrap());
    context.insert("dir_trail", &ancestors);
    context.insert("dir_tree", &dir_tree);

    Ok(Html(tera.render("directory.tera", &context).unwrap()))
}


#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct CreateDirectory {
    name: String
}

#[axum::debug_handler]
pub async fn create_directory(
  State(state): State<AppState>,
  Path(dir_id): Path<i64>,
  Form(dir): Form<CreateDirectory>
) -> Result<impl IntoResponse, Error> {

  let new_dir = state.create_directory(dir_id, dir.name).await?;

  Ok(Redirect::to(&format!("/dir/{}", new_dir.id)))
}
pub async fn upload_image(
    State(state): State<AppState>,
    Path(dir_id): Path<i64>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, Error> {
    let mut name = String::new();
    while let Some(field) = multipart.next_field().await? {
        if field.name().is_some_and(|n| n == ("name")) {
            name = field.text().await?;
            continue;
        }

        let file_name = if name.is_empty() {
            field.file_name().ok_or(Error::StatusCode(
                StatusCode::BAD_REQUEST,
                "Image has no file_name".to_owned(),
            ))?
        } else {
            &name
        }
        .to_string();
        let data = field.bytes().await?;

        let image = Image::new(dir_id, file_name, String::new());
        state.create_image(image, data).await?;
    }
    Ok(Redirect::to(&format!("/dir/{dir_id}")))
}

pub async fn delete_images(
    state: State<AppState>,
    Path(dir_id): Path<i64>,
    Form(checkboxes): Form<BTreeMap<String, String>>,
) -> Result<impl IntoResponse, Error> {
    let ids = checkboxes
        .into_iter()
        .filter_map(|(id, check)| {
            if let Ok(id) = id.parse() {
                if check == "on" {
                    return Some(id);
                }
            }
            None
        })
        .collect();

    state.delete_image_many(&ids).await?;

    Ok(Redirect::to(&format!("/dir/{dir_id}")))
}

pub async fn move_images(
    state: State<AppState>,
    Path(_): Path<i64>,
    Form(checkboxes): Form<BTreeMap<String, String>>,
) -> Result<impl IntoResponse, Error> {
    let new_dir_id: i64 = checkboxes
        .get("new_dir_id")
        .and_then(|id| id.parse().ok())
        .ok_or(Error::StatusCode(
            StatusCode::BAD_REQUEST,
            format!("Missing new_dir_id."),
        ))?;

    let ids = checkboxes
        .into_iter()
        .filter_map(|(id, check)| {
            if let Ok(id) = id.parse() {
                if check == "on" {
                    return Some(id);
                }
            }
            None
        })
        .collect();

    state.move_images(&ids, new_dir_id).await?;

    Ok(Redirect::to(&format!("/dir/{new_dir_id}")))
}
