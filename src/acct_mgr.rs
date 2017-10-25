extern crate std;

use common;

pub enum CookieStatus {
    None,
    Required(String),
}

impl Default for CookieStatus {
    fn default() -> Self {
        CookieStatus::None
    }
}

#[derive(Default)]
pub struct AcctMgrInfo {
    pub master_url: String,
    pub project_name: String,
    pub login_name: Option<String>,
    pub user_name: String,
    pub password_hash: String,
    pub cookie_status: CookieStatus,
    pub password_error: bool,
}

impl common::ProjAm for AcctMgrInfo {
    fn master_url(&self) -> String {
        self.master_url.clone()
    }

    fn project_name(&self) -> Option<String> {
        Some(self.project_name.clone())
    }
}
