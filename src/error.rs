use axum::{extract::multipart::MultipartError, http::StatusCode, response::IntoResponse};
use tokio::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not find row with this id.")]
    NoSuchEntity,
    #[error("{1}")]
    StatusCode(StatusCode, String),
    #[error("Database Error: {0}")]
    Database(sqlx::Error),
    #[error("Multipart Error: {0}")]
    Multipart(MultipartError),
    #[error("IO error: {0}")]
    Io(io::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (
            match self {
                Error::NoSuchEntity => StatusCode::NOT_FOUND,
                Error::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
                Error::Multipart(_) => StatusCode::BAD_REQUEST,
                Error::StatusCode(status, _) => status,
                Error::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
            self.to_string(),
        )
            .into_response()
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<MultipartError> for Error {
    fn from(value: MultipartError) -> Self {
        Error::Multipart(value)
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::NoSuchEntity,
            // sqlx::Error::Configuration(error) => todo!(),
            // sqlx::Error::Database(database_error) => todo!(),
            // sqlx::Error::Io(error) => todo!(),
            // sqlx::Error::Tls(error) => todo!(),
            // sqlx::Error::Protocol(_) => todo!(),
            // sqlx::Error::TypeNotFound { type_name } => todo!(),
            // sqlx::Error::ColumnIndexOutOfBounds { index, len } => todo!(),
            // sqlx::Error::ColumnNotFound(_) => todo!(),
            // sqlx::Error::ColumnDecode { index, source } => todo!(),
            // sqlx::Error::Encode(error) => todo!(),
            // sqlx::Error::Decode(error) => todo!(),
            // sqlx::Error::AnyDriverError(error) => todo!(),
            // sqlx::Error::PoolTimedOut => todo!(),
            // sqlx::Error::PoolClosed => todo!(),
            // sqlx::Error::WorkerCrashed => todo!(),
            // sqlx::Error::Migrate(migrate_error) => todo!(),
            err => Self::Database(err),
        }
    }
}
