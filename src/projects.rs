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

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use self::treexml_util::{make_tree_element, make_text_element};

/// Split off from Project for mutability reasons.
#[derive(Default)]
pub struct ProjectData {
    pub project_name: Option<String>,
    pub apps: HashMap<uuid::Uuid, app::App>,
    pub project_prefs: Option<treexml::Element>,
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


impl ProjectData {
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

    pub fn parse_account(&mut self, v: &treexml::Element) -> errors::Result<()> {
        Ok(())
    }

    pub fn write_account_file(&self) -> errors::Result<()> {
        Ok(())
    }
}

#[derive(Default)]
pub struct Project {
    _master_url: String,
    pub data: context::Context<ProjectData>,
}

impl Hash for Project {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self._master_url.hash(state);
    }
}
impl PartialEq for Project {
    fn eq(&self, other: &Self) -> bool {
        self._master_url == other._master_url
    }
}
impl Eq for Project {}

impl common::ProjAm for Project {
    fn master_url(&self) -> String {
        self._master_url.clone()
    }

    fn project_name(&self) -> Option<String> {
        self.data.await_force(
            |project| project.project_name.clone(),
        )
    }
}

impl<'a> From<&'a Project> for treexml::Element {
    fn from(v: &Project) -> treexml::Element {
        make_tree_element(
            "project",
            vec![
                make_text_element("master_url", v.master_url()),
                make_text_element("project_name", v.get_project_name()),
                // TODO: serialize more fields
            ],
        )
    }
}

impl Project {
    pub fn new(master_url: String) -> Project {
        Project {
            _master_url: master_url,
            ..Default::default()
        }
    }

    pub fn project_dir(&self) -> std::path::PathBuf {
        let mut v = std::path::PathBuf::new();
        v.push(file_names::PROJECTS_DIR);
        v.push(self.master_url());
        v
    }

    pub fn make_project_dir(&self) -> errors::Result<()> {
        Ok(())
    }
}

pub struct Projects {
    pub data: HashSet<Project>,
    clock_source: common::ClockSource,
    logger: messages::SafeLogger,
}

impl Projects {
    pub fn new(clock_source: common::ClockSource, logger: messages::SafeLogger) -> Projects {
        Projects {
            data: Default::default(),
            clock_source: clock_source,
            logger: logger,
        }
    }

    pub fn find_by_url(&self, url: &str) -> Option<&Project> {
        self.data.get(&Project {
            _master_url: url.to_string(),
            ..Default::default()
        })
    }
}
