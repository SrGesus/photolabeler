use axum::{extract::State, http::StatusCode};

use crate::{error::Error, state::AppState};

pub async fn refresh(State(state): State<AppState>) -> Result<StatusCode, Error> {
    let directories = state.get_directory_parentless().await?;
    for dir in directories {
        state.delete_missing(&dir).await?;
        state.add_missing(dir).await?;
    }
    Ok(StatusCode::NO_CONTENT)
}
