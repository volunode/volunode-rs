extern crate chan;
extern crate std;
extern crate treexml;
extern crate uuid;

use acct_setup;
use app;
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
use util;

use std::io::Write;
use std::ops::Deref;

use common::ProjAm;

use std::collections::HashMap;
use std::sync::Arc;

pub struct ClientState {
    pub clock_source: common::ClockSource,

    pub cc_config: cc_config::CCConfig,
    pub messages: messages::SafeLogger,

    pub host_info: hostinfo::HostInfo,
    pub projects: projects::Projects,
    pub file_infos: HashMap<uuid::Uuid, file_info::FileInfo>,

    pub project_attach: acct_setup::ProjectAttach,
    pub project_init: project_init::ProjectInit,
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
        Self {
            messages: Arc::new(messages::DummyLogger::default()),
            ..Default::default()
        }
    }
}

struct AppInfo {
    file_infos: Vec<file_info::FileInfo>,
    apps: Vec<app::App>,
    app_versions: Vec<app::AppVersion>,
}

impl ClientState {
    pub fn new(messages: messages::SafeLogger) -> Self {
        {
            let clock_source = common::system_clock_source();
            let mut v = Self {
                clock_source: clock_source.clone(),
                messages: messages.clone(),
                projects: projects::Projects::new(clock_source.clone(), messages.clone()),
                ..Default::default()
            };

            v
        }
    }

    pub fn write_state_file(&self) -> Result<(), std::io::Error> {
        std::fs::File::create(constants::STATE_FILE_NAME)?
            .write_fmt(format_args!("{}", treexml::Element::from(self)))?;
        Ok(())
    }

    pub fn parse_app_info(&mut self, node: &treexml::Element) -> errors::Result<()> {
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
                format!("Adding projects is not allowed"),
            ));
        }

        let canonical_master_url = util::canonicalize_url(url);

        if false {
            bail!(errors::ErrorKind::InvalidURLError(
                format!("Invalid master URL"),
            ));
        }

        let auth = _auth.trim().to_string();
        if auth.is_empty() {
            bail!(errors::ErrorKind::AuthError(format!("Missing account key")));
        }

        if self.projects.find_by_url(&canonical_master_url).is_some() {
            bail!(errors::ErrorKind::AlreadyAttachedError(format!(
                "Already attached to project {}",
                &canonical_master_url
            )));
        }

        let mut proj = projects::Project::new(canonical_master_url.to_string());

        let project_dir = proj.project_dir();

        proj.data.await_mut_force({
            let p_name = Arc::new(project_name.to_string());
            let p_auth = Arc::new(auth.to_string());
            let attached_via_acct_mgr = attached_via_acct_mgr;
            let canonical_master_url = canonical_master_url.clone();
            move |project| -> errors::Result<()> {
                project.project_name = Some((*p_name).clone());
                project.authenticator = (*p_auth).clone();
                project.attached_via_acct_mgr = attached_via_acct_mgr;

                project.write_account_file()?;

                project.parse_account(
                    &treexml::Document::parse(std::fs::File::open(
                        file_names::account_filename(&canonical_master_url),
                    )?)?
                        .root
                        .unwrap(),
                )?;

                Ok(())
            }
        })?;

        let path = std::path::PathBuf::from(&format!(
            "{}/{}",
            project_dir.display(),
            file_names::APP_INFO_FILE_NAME
        ));
        if path.exists() {
            proj.data.await_mut_force(
                |project| project.anonymous_platform = true,
            );
            std::fs::File::open(path).map(|f| {
                treexml::Document::parse(f).map(|doc| {
                    doc.root.map(|e| { self.parse_app_info(&e); });
                });
            });
        } else {
            std::fs::remove_dir_all(project_dir);
        }

        proj.make_project_dir()?;

        proj.data.await_mut_force(|project| {
            project.sched_rpc_pending = Some(common::RpcReason::Init);
        });

        assert!(self.projects.data.insert(proj));

        self.set_client_state_dirty("Add project");

        Ok(())
    }
}
