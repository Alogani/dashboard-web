use askama::Template;

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub error_message: &'static str,
    pub welcome_message: String,
}

#[derive(Template)]
#[template(path = "login_error.html")]
pub struct LoginError<'a> {
    pub message: &'a str,
}
