pub const APP_INFO_FILE_NAME: &str = "app_info.xml";
pub const PROJECT_INIT_FILE_NAME: &str = "project_init.xml";

pub const PROJECTS_DIR: &str = "projects";

pub fn account_filename(canonical_master_url: &str) -> String {
    format!("account_{}", canonical_master_url)
}
