extern crate chrono;
extern crate crypto;
extern crate treexml;
extern crate treexml_util;
extern crate uuid;

use app;
use common;
use messages;

use common::ProjAm;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct Project {
    _master_url: String,
    _project_name: Option<String>,
    pub apps: Vec<app::App>,
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
    pub user_create_time: common::Time,
    pub userid: u64,
    pub teamid: u64,
    pub hostid: u64,
    pub host_total_credit: f64,
    pub host_expavg_credit: f64,
    pub host_create_time: common::Time,
    pub last_rpc_time: common::Time,
    pub cpu_ec: f64,
    pub cpu_time: f64,
    pub gpu_ec: f64,
    pub gpu_time: f64,

    pub rpc_seqno: usize,
    pub nrpc_failures: usize,
    pub master_fetch_failures: usize,
    pub min_rpc_time: common::Time,
    pub next_rpc_time: common::Time,
    pub master_url_fetch_pending: bool,

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
        treexml::Element {
            name: "project".into(),
            children: vec![
                treexml_util::serialize_node("master_url", v.master_url()),
                treexml_util::serialize_node("project_name", v.get_project_name()),
            ],
            ..Default::default()
        }
    }
}

impl Project {
    pub fn can_request_work(&self, now: common::ClockSource) -> bool {
        !(self.suspended_via_gui || self.master_url_fetch_pending || self.min_rpc_time > now() ||
              self.dont_request_more_work)
    }

    pub fn start_computation(&mut self) {
        self.apps.iter_mut().map(|app| {});
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

    pub fn find_project<F: Fn(&Project)>(&self, k: &str, f: F) {
        self.data.iter().map(
            |proj| if proj.get_project_name() == k {
                f(proj);
            },
        );
    }

    pub fn find_project_mut<F: Fn(&mut Project)>(&mut self, k: &str, f: F) {
        self.data.iter_mut().map(
            |proj| if proj.get_project_name() == k {
                f(proj);
            },
        );
    }
}
