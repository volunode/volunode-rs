extern crate chan;
extern crate std;
extern crate treexml;
extern crate uuid;

use acct_mgr;
use acct_setup;
use cc_config;
use common;
use constants;
use errors;
use file_info;
use file_names;
use hostinfo;
use messages;
use project_init;
use projects;
use tasks;
use util;

use std::io::Write;
use std::ops::Deref;

use std::collections::HashMap;
use std::sync::Arc;

use common::*;

#[derive(Clone)]
pub struct RunSettings {
    clock_source: Arc<ClockSource>,
    perm_mode: RunMode,
    temp_mode: Option<(RunMode, Time)>,
    prev_mode: RunMode,
}

impl RunSettings {
    pub fn get_perm(&self) -> RunMode {
        self.perm_mode
    }

    pub fn get_prev(&self) -> RunMode {
        self.prev_mode
    }

    pub fn get_current(&self) -> RunMode {
        if let Some(&(temp_mode, until)) = self.temp_mode.as_ref() {
            if self.clock_source.now() < until {
                return temp_mode
            }
        }

        self.perm_mode
    }
    pub fn delay(&self) -> Duration {
        Duration::seconds(match self.temp_mode {
            Some((mode, end)) => std::cmp::max(end.timestamp() - self.clock_source.now().timestamp(), 0),
            None => 0,
        })
    }
}

impl ClockInitializable for RunSettings {
    fn new_with_clock(clock_source: Arc<ClockSource>) -> Self {
        Self {
            clock_source: clock_source,
            perm_mode: Default::default(),
            temp_mode: Default::default(),
            prev_mode: Default::default(),
        }
    }
}

pub struct ClientState {
    pub clock_source: Box<ClockSource>,

    pub cc_config: cc_config::CCConfig,
    pub messages: messages::SafeLogger,

    pub host_info: hostinfo::HostInfo,
    pub projects: projects::Projects,
    pub file_infos: HashMap<uuid::Uuid, file_info::FileInfo>,

    pub project_attach: acct_setup::ProjectAttach,
    pub project_init: Option<project_init::ProjectInit>,

    pub acct_mgr_info: acct_mgr::AcctMgrInfo,

    pub tasks: Box<tasks::TaskServer + Send + Sync + 'static>,

    pub run_mode: RunSettings,
    pub gpu_run_mode: RunSettings,

    pub suspend_reason: Option<RpcReason>,
    pub gpu_suspend_reason: Option<RpcReason>,
}

impl<'a> From<&'a ClientState> for treexml::Element {
    fn from(v: &ClientState) -> treexml::Element {
        let host_info = &v.host_info;
        let projects = &v.projects;

        treexml::Element {
            name: "client_state".into(),
            children: {
                let mut v = Vec::new();
                v.push(host_info.deref().into());
                v.append(&mut projects
                    .deref()
                    .data
                    .iter()
                    .map(|v| v.into())
                    .collect());
                v
            },
            ..Default::default()
        }
    }
}

impl Default for ClientState {
    fn default() -> Self {
        let messages: messages::SafeLogger = Arc::new(messages::DummyLogger::default());
        let clock_source = Arc::new(SystemClockSource);
        Self {
            clock_source: Box::new(*clock_source.clone()),
            messages: Arc::clone(&messages),
            projects: projects::Projects::new(Arc::clone(&messages)),
            tasks: Box::new(tasks::MockTaskServer::default()),

            cc_config: Default::default(),
            host_info: Default::default(),
            file_infos: Default::default(),
            project_attach: Default::default(),
            project_init: Default::default(),

            acct_mgr_info: Default::default(),

            gpu_run_mode: ClockInitializable::new_with_clock(clock_source.clone()),
            run_mode: ClockInitializable::new_with_clock(clock_source.clone()),

            suspend_reason: Default::default(),
            gpu_suspend_reason: Default::default(),
        }
    }
}

impl ClientState {
    pub fn new(messages: messages::SafeLogger) -> Self {
        Self {
            messages: Arc::clone(&messages),
            ..Default::default()
        }

    }

    pub fn write_state_file(&self) -> errors::Result<()> {
        std::fs::File::create(constants::STATE_FILE_NAME)?
            .write_fmt(format_args!("{}", treexml::Element::from(self)))?;
        Ok(())
    }

    pub fn parse_app_info(
        &mut self,
        project: &projects::Project,
        root: &treexml::Element,
    ) -> errors::Result<()> {
        for node in &root.children {
            match node.name.as_str() {
                "file" | "file_info" => {
                    let fi = file_info::FileInfo::from(node);
                    if !fi.download_urls.is_empty() || !fi.upload_urls.is_empty() {
                        bail!(errors::ErrorKind::XMLError("".into()));
                    }

                    let _ = project;

                    util::insert_unique(&mut self.file_infos, fi);
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn sort_projects_by_name(&mut self) {}

    pub fn set_client_state_dirty(&mut self, _: &str) {}

    pub fn add_project(
        &mut self,
        url: &str,
        _auth: &str,
        project_name: &str,
        attached_via_acct_mgr: bool,
    ) -> errors::Result<()> {
        if self.cc_config.disallow_attach {
            bail!(errors::ErrorKind::UserPermissionError(
                "Adding projects is not allowed".into(),
            ));
        }

        let canonical_master_url = util::canonicalize_url(url);

        if false {
            bail!(errors::ErrorKind::InvalidURLError(
                "Invalid master URL".into(),
            ));
        }

        let auth = _auth.trim().to_string();
        if auth.is_empty() {
            bail!(errors::ErrorKind::AuthError("Missing account key".into()));
        }

        if self.projects.find_by_url(&canonical_master_url).is_some() {
            bail!(errors::ErrorKind::AlreadyAttachedError(format!(
                "Already attached to project {}",
                &canonical_master_url
            )));
        }

        let proj = projects::Project::new(canonical_master_url.to_string());

        let project_dir = proj.project_dir();

        {
            let mut p = proj.data.lock().unwrap();
            p.project_name = Some(project_name.into());
            p.authenticator = auth.clone();
            p.attached_via_acct_mgr = attached_via_acct_mgr;

            p.write_account_file()?;

            p.parse_account(
                &treexml::Document::parse(std::fs::File::open(
                    file_names::account_filename(&canonical_master_url),
                )?)?
                    .root
                    .unwrap(),
            )?;
        }

        let path = std::path::PathBuf::from(&format!(
            "{}/{}",
            project_dir.display(),
            file_names::APP_INFO_FILE_NAME
        ));
        if path.exists() {
            proj.data.lock().unwrap().anonymous_platform = true;
            let _ = std::fs::File::open(path).map(|f| {
                let _ = treexml::Document::parse(f).map(|doc| {
                    doc.root.map(
                        |e| { let _ = self.parse_app_info(&proj, &e); },
                    );
                });
            });
        } else {
            let _ = std::fs::remove_dir_all(project_dir);
        }

        proj.make_project_dir()?;

        proj.data.lock().unwrap().sched_rpc_pending = Some(RpcReason::Init);

        assert!(self.projects.data.insert(proj));

        self.set_client_state_dirty("Add project");

        Ok(())
    }
}
