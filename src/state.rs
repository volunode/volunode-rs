extern crate chan;
extern crate std;
extern crate treexml;

use acct_setup;
use cc_config;
use common;
use constants;
use errors;
use file_names;
use hostinfo;
use messages;
use project_init;
use projects;

use std::io::Write;
use std::ops::Deref;

use common::ProjAm;

use std::sync::Arc;

pub struct ClientState {
    pub clock_source: common::ClockSource,

    pub cc_config: cc_config::CCConfig,
    pub messages: messages::SafeLogger,

    pub host_info: hostinfo::HostInfo,
    pub projects: projects::Projects,

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

    pub fn parse_app_info(&mut self, _: &treexml::Element) -> errors::Result<()> {
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

        let canonical_master_url = url.to_string();

        if false {
            bail!(errors::ErrorKind::InvalidURLError(
                format!("Invalid master URL"),
            ));
        }

        let auth = _auth.trim();
        if auth.is_empty() {
            bail!(errors::ErrorKind::AuthError(format!("Missing account key")));
        }

        for p in &self.projects.data {
            if p.master_url() == canonical_master_url {
                bail!(errors::ErrorKind::AlreadyAttachedError(format!(
                    "Already attached to project {}",
                    &canonical_master_url
                )));
            }
        }

        let mut project =
            projects::Project::new(canonical_master_url.to_string(), Some(project_name.into()));

        project.authenticator = auth.into();
        project.attached_via_acct_mgr = attached_via_acct_mgr;

        project.write_account_file()?;

        project.parse_account(
            &treexml::Document::parse(std::fs::File::open(
                file_names::account_filename(&canonical_master_url),
            )?)?
                .root
                .unwrap(),
        )?;

        let path = std::path::PathBuf::from(&format!(
            "{}/{}",
            project.project_dir().display(),
            file_names::APP_INFO_FILE_NAME
        ));
        if path.exists() {
            project.anonymous_platform = true;
            std::fs::File::open(path).map(|f| {
                treexml::Document::parse(f).map(|doc| {
                    doc.root.map(|e| { self.parse_app_info(&e); });
                });
            });
        } else {
            std::fs::remove_dir_all(project.project_dir());
        }

        project.make_project_dir()?;

        project.sched_rpc_pending = Some(common::RpcReason::Init);
        self.projects.data.push(project);
        self.sort_projects_by_name();

        self.set_client_state_dirty("Add project");

        Ok(())
    }
}
