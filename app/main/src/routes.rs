use crate::templates::MainLanding;
use askama::Template;
use axum::response::Html;

pub async fn landing_page() -> Html<String> {
    Html(MainLanding.render().unwrap())
}
