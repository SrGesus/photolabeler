

pub fn router() -> Router<AppState> {
  Router::new()
      .route("/", get(page))
      .route("/", post(register_folder))
      .route("/:id/delete", post(unregister_folder))
}





