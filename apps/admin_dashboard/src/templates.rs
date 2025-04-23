use askama::Template;

#[derive(Template)]
#[template(path = "admin_panels.html")]
pub struct AdminPanels<'a> {
    pub panels: Vec<(&'a str, Vec<(&'a str, &'a str)>)>,
}

#[derive(Template)]
#[template(path = "command_result.html")]
pub struct RouterAdminCommandResult<'a> {
    pub cmd: &'a str,
    pub output: String,
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct RouterAdminError<'a> {
    pub message: &'a str,
}
