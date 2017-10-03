extern crate chan;
extern crate std;
extern crate treexml;
extern crate uuid;

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
use util;

use std::io::Write;
use std::ops::Deref;

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
    pub project_init: Option<project_init::ProjectInit>,
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
        let clock_source = common::system_clock_source();
        let messages: messages::SafeLogger = Arc::new(messages::DummyLogger::default());
        Self {
            clock_source: Arc::clone(&clock_source),
            messages: Arc::clone(&messages),
            projects: projects::Projects::new(Arc::clone(&clock_source), Arc::clone(&messages)),

            cc_config: Default::default(),
            host_info: Default::default(),
            file_infos: Default::default(),
            project_attach: Default::default(),
            project_init: Default::default(),
        }
    }
}

impl ClientState {
    pub fn new(messages: messages::SafeLogger) -> Self {
        let clock_source = common::system_clock_source();
        Self {
            clock_source: Arc::clone(&clock_source),
            messages: Arc::clone(&messages),
            projects: projects::Projects::new(Arc::clone(&clock_source), Arc::clone(&messages)),
            ..Default::default()
        }

    }

    pub fn write_state_file(&self) -> Result<(), std::io::Error> {
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

        proj.data.await_mut_force(|project| {
            project.sched_rpc_pending = Some(common::RpcReason::Init);
        });

        assert!(self.projects.data.insert(proj));

        self.set_client_state_dirty("Add project");

        Ok(())
    }
}
