extern crate std;

/// Represents the contents of project_init.xml, specifying an account to attach to initially
#[derive(Debug)]
pub struct ProjectInit {
    pub url: String,
    pub name: String,
    pub team_name: String,
    pub account_key: String,

    pub setup_cookie: Vec<u8>,

    pub embedded: bool,
}
