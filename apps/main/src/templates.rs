use askama::Template;

#[derive(Template)]
#[template(path = "landing.html")]
pub struct LandingPage {
    pub external_links: Vec<(String, String)>,
}
