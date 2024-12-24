use sqlx::{migrate, SqlitePool};

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            dotenvy::dotenv().ok();

            let url = std::env::var("DATABASE_URL").unwrap();
            let pool = SqlitePool::connect(&url).await.unwrap();
            migrate!("../migrations").run(&pool).await.unwrap();
        })
}
