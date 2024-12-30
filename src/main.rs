use std::sync::Arc;

use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use db::Database;
use serde::{Deserialize, Serialize};

mod backend;
mod db;
mod frontend;
mod error;

#[derive(Clone)]
struct AppState(Arc<Database>);

const SERVER_ADDRESS: &str = "0.0.0.0:3071";
const DATABASE_URL: &str = "sqlite:./dev.sqlite?mode=rwc";

/// Get the value of the environment variable PHOTOLABELER_"$A", or defaults to $A value
macro_rules! env_var {
    ($A:ident) => {
        std::env::var(concat!("PHOTOLABELER", "_", stringify!($A))).unwrap_or($A.to_string())
    };
}

#[tokio::main]
async fn main() {
    // Read environment variables from .env
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    let db_url = env_var!(DATABASE_URL);

    // let mut conn = sqlx::SqliteConnection::connect(&db_url).await.unwrap();
    let db = db::Database::new(&db_url).await.unwrap();
    let state = AppState(Arc::new(db));

    // Get address that server should be bound to.
    let addr = env_var!(SERVER_ADDRESS);

    let api = backend::router();

    let app = Router::new()
        .route("/", get(frontend::homepage))
        .route("/:directory_id", get(frontend::images))
        .route("/image/:image_id", get(frontend::image))
        .nest("/", api)
        .with_state(state);

    // Start up our server
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("Serving Photolabeler in http://{}", &addr);
    axum::serve(listener, app).await.unwrap();
}
