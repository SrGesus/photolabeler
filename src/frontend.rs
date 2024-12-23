use axum::response::Html;
use tera::{Context, Tera};



#[axum::debug_handler]
pub async fn homepage() -> Html<String> {
  let tera = Tera::new("templates/**/*.html.tera").unwrap();
  let context = Context::new();

  axum::response::Html(tera.render("layout.html.tera", &context).unwrap())
}
