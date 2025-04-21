use askama::Template;

#[derive(Template)]
#[template(path = "landing.html")]
pub struct MainLanding;
