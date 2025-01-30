use state::AppState;
use tower_http::trace::TraceLayer;

use axum::{
    body::{Body, Bytes},
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
};

use http_body_util::BodyExt;

mod error;
mod routes;
mod state;

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

    let state = AppState::new(&db_url).await.unwrap();

    // Get address that server should be bound to.
    let addr = env_var!(SERVER_ADDRESS);

    let app = routes::router()
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(
            middleware::from_fn(print_request_response)
        );

    // Start up our server
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("Serving Photolabeler in http://{}/", &addr);
    axum::serve(listener, app).await.unwrap();
}

async fn print_request_response(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{direction} body = {body:?}");
    }

    Ok(bytes)
}
