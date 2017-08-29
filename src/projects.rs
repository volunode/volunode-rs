extern crate uuid;

use app;
use common;
use messages;

use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Project {
    _master_url: String,
    _project_name: Option<String>,
    pub apps: HashMap<uuid::Uuid, app::App>,
}

impl common::ProjAm for Project {
    fn master_url(&self) -> &str {
        &self._master_url
    }

    fn project_name(&self) -> Option<&str> {
        self._project_name.as_ref().map(|s| s.as_str())
    }
}

pub struct Projects {
    pub data: Vec<Project>,
    clock_source: common::ClockSource,
    logger: messages::SafeLogger,
}

impl Projects {
    pub fn new(clock_source: common::ClockSource, logger: messages::SafeLogger) -> Projects {
        Projects {
            data: Vec::new(),
            clock_source: clock_source,
            logger: logger,
        }
    }
}
