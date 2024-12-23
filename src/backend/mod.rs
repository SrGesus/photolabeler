use axum::{extract::State, routing::post, Router};
use std::{fs, sync::Arc};

use crate::{
    db::{directory::Directory, image::Image, Database},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
    .route("/refresh", post(refresh_directories))
}

pub async fn refresh_directories(State(AppState(database)): State<AppState>) {
    let directories = Directory::get_all(&database).await.unwrap();
    for dir in directories {
        for file in fs::read_dir(dir.path()).unwrap() {
            let file = file.unwrap();
            let name = file.file_name();
            let i = Image::new(*dir.id(), name.into_string().unwrap(), None);
            i.insert(&database).await.ok();
        }
    }
}
