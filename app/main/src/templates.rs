use askama::Template;
use std::collections::HashMap;

#[derive(Template)]
#[template(path = "landing.html")]
pub struct LandingPage {
    pub external_links: HashMap<String, String>,
}
