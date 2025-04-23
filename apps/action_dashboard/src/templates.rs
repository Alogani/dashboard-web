use askama::Template;
use config::admin_config::AdminConsole;

#[derive(Template)]
#[template(path = "action_dashboard.html")]
pub struct AdminConsoleView<'a> {
    pub console: &'a AdminConsole,
}

#[derive(Template)]
#[template(path = "action_result.html")]
pub struct ActionResult<'a> {
    pub action_name: &'a str,
    pub output: String,
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ExecutionError<'a> {
    pub message: &'a str,
}
