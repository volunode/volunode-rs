extern crate uuid;

use workunit;

use self::uuid::Uuid;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct AppVersion {
    pub file_name: String,
    pub app_name: String,
    pub version_num: i32,
    pub platform: String,
    pub plan_class: String,
    pub api_version: String,
    pub avg_ncpus: f64,
    pub max_ncpus: f64,
}

#[derive(Debug)]
pub struct App {
    pub name: String,
    pub user_friendly_name: String,
    pub work_units: HashMap<Uuid, workunit::Workunit>,
    pub versions: HashMap<Uuid, AppVersion>,
    pub active_tasks: HashSet<Uuid>,
}
