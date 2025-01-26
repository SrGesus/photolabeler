use state::AppState;

// mod backend;
mod error;
mod routes;
mod state;

// #[derive(Clone)]
// struct AppState(Arc<Database>);

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
    // let db = db::Database::new(&db_url).await.unwrap();
    // let state = AppState(Arc::new(db));
    let state = AppState::new(&db_url).await.unwrap();

    // Get address that server should be bound to.
    let addr = env_var!(SERVER_ADDRESS);

    let app = routes::router().with_state(state);

    // Start up our server
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("Serving Photolabeler in http://{}/", &addr);
    axum::serve(listener, app).await.unwrap();
}
