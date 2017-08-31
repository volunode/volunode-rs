extern crate treexml;
extern crate uuid;

use app;
use common;
use messages;

use common::ProjAm;

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

impl<'a> From<&'a Project> for treexml::Element {
    fn from(v: &Project) -> treexml::Element {
        treexml::ElementBuilder::new("project")
            .children(vec![
                treexml::ElementBuilder::new("master_url").text(
                    v.master_url()
                ),
                treexml::ElementBuilder::new("project_name").text(
                    v.get_project_name()
                ),
            ])
            .element()
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
