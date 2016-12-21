extern crate uuid;

use workunit;

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct AppVersion {
    pub app_name: String,
    pub version_num: i32,
    pub platform: String,
    pub plan_class: String,
    pub api_version: String,
    pub avg_ncpus: f64,
    pub max_ncpus: f64,
}

#[derive(Clone, Debug)]
pub struct App {
    pub name: String,
    pub user_friendly_name: String,
    pub work_units: HashMap<uuid::Uuid, workunit::Workunit>,
    pub versions: HashMap<uuid::Uuid, AppVersion>,
}
