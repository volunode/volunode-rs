extern crate std;

extern crate chrono;
extern crate crypto;
extern crate treexml;
extern crate treexml_util;
extern crate uuid;

use app;
use common;
use context;
use errors;
use file_info;
use file_names;
use messages;

use common::ProjAm;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use self::treexml_util::{make_tree_element, make_text_element};

#[derive(Default)]
pub struct Project {
    _master_url: String,
    _project_name: Option<String>,
    pub apps: HashMap<uuid::Uuid, app::App>,
    pub project_prefs: Option<treexml::Element>,
    pub file_infos: HashMap<uuid::Uuid, file_info::FileInfo>,
    pub host_venue: String,
    pub scheduler_urls: Vec<String>,
    pub user_name: String,
    pub team_name: String,
    pub email_hash: String,
    pub cross_project_id: String,
    pub external_cpid: String,
    pub user_total_credit: f64,
    pub user_expavg_credit: f64,
    pub user_create_time: Option<common::Time>,
    pub userid: u64,
    pub teamid: u64,
    pub hostid: u64,
    pub host_total_credit: f64,
    pub host_expavg_credit: f64,
    pub host_create_time: Option<common::Time>,
    pub last_rpc_time: Option<common::Time>,
    pub cpu_ec: f64,
    pub cpu_time: f64,
    pub gpu_ec: f64,
    pub gpu_time: f64,

    pub rpc_seqno: usize,
    pub nrpc_failures: usize,
    pub master_fetch_failures: usize,
    pub min_rpc_time: Option<common::Time>,
    pub next_rpc_time: Option<common::Time>,
    pub master_url_fetch_pending: bool,
    pub sched_rpc_pending: Option<common::RpcReason>,

    pub anonymous_platform: bool,
    pub attached_via_acct_mgr: bool,
    pub authenticator: String,

    pub disk_usage: f64,

    pub suspended_via_gui: bool,
    pub dont_request_more_work: bool,
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
        make_tree_element(
            "project",
            vec![
                make_text_element("master_url", v.master_url()),
                make_text_element("project_name", v.get_project_name()),
            ],
        )
    }
}

impl Project {
    pub fn new(master_url: String, project_name: Option<String>) -> Project {
        Project {
            _master_url: master_url,
            _project_name: project_name,
            ..Default::default()
        }
    }

    pub fn can_request_work(&self, now: &common::Time) -> bool {
        !(self.suspended_via_gui || self.master_url_fetch_pending ||
              {
                  if let &Some(ref v) = &self.min_rpc_time {
                      v > now
                  } else {
                      false
                  }
              } || self.dont_request_more_work)
    }

    pub fn start_computation(&mut self) {
        self.apps.iter_mut().map(|app| {});
    }

    pub fn set_project_name(&mut self, v: Option<String>) {
        self._project_name = v;
    }

    pub fn project_dir(&self) -> std::path::PathBuf {
        let mut v = std::path::PathBuf::new();
        v.push(file_names::PROJECTS_DIR);
        v.push(self.master_url());
        v
    }

    pub fn make_project_dir(&mut self) -> errors::Result<()> {
        Ok(())
    }

    pub fn parse_account(&mut self, v: &treexml::Element) -> errors::Result<()> {
        Ok(())
    }

    pub fn write_account_file(&self) -> errors::Result<()> {
        Ok(())
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
