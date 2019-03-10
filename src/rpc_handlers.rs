use futures::future;
use futures::prelude::*;
use futures::prelude::*;
use std;
use std::collections::*;
use std::io::Read;
use std::sync::{Arc, Mutex};
use treexml;
use treexml_util;
use treexml_util::Unmarshaller;
use treexml_util::{make_text_element, make_tree_element};

use acct_mgr;
use common;
use common::ProjAm;
use constants;
use errors;
use projects;
use state;
use util;

pub type ElementFuture = Box<Future<Item = Option<treexml::Element>, Error = errors::Error>>;

fn make_error(v: &str) -> treexml::Element {
    make_text_element("error", v)
}

/// Contains handlers to RPC calls
#[derive(Clone)]
pub struct H {
    pub state: Arc<Mutex<state::ClientState>>,
    pub incoming: treexml::Element,
}

impl H {
    pub fn get_message_count(self) -> ElementFuture {
        Box::new(
            util::mutex_critical(self.state, |state| state.messages.len())
                .map(|v| Some(make_text_element("seqno", v))),
        )
    }

    pub fn get_messages(self) -> ElementFuture {
        let seqno = treexml_util::find_value::<usize>("seqno", &self.incoming)
            .ok()
            .unwrap_or(None);

        Box::new(
            util::mutex_critical(self.state, move |state| Ok(state.messages.to_xml(seqno)))
                .map(|v| Some(v)),
        )
    }

    pub fn get_notices(self) -> ElementFuture {
        Box::new(future::ok(Some(treexml::Element::new("notices"))))
    }

    pub fn get_state(self) -> ElementFuture {
        Box::new(
            util::mutex_critical(self.state, move |state| Ok(treexml::Element::from(&*state)))
                .map(|v| Some(v)),
        )
    }

    pub fn get_all_projects_list(self) -> ElementFuture {
        Box::new(
            std::fs::File::open(constants::ALL_PROJECTS_LIST_FILENAME)
                .ok()
                .and_then(|mut file| {
                    let mut s = String::new();
                    file.read_to_string(&mut s).ok().and_then(|_| {
                        treexml::Document::parse(std::io::Cursor::new(format!(
                            "<root>{}</root>",
                            &s
                        )))
                        .ok()
                        .map(|doc| make_tree_element("projects", doc.root.unwrap().children))
                    })
                }),
        )
    }

    pub fn get_disk_usage(self) -> ElementFuture {
        Box::new(
            util::mutex_critical(self.state, |state| {
                let mut out = Vec::new();
                for proj in &state.projects.data {
                    out.push({
                        treexml::Element {
                            name: "project".into(),
                            children: vec![
                                make_text_element("master_url", proj.master_url()),
                                make_text_element(
                                    "disk_usage",
                                    proj.data.lock().unwrap().disk_usage,
                                ),
                            ],
                            ..Default::default()
                        }
                    });
                }

                out.append(&mut vec![
                    make_text_element("d_total", &state.host_info.d_total),
                    make_text_element("d_free", &state.host_info.d_free),
                    make_text_element("d_boinc", &state.host_info.d_boinc),
                    make_text_element("d_allowed", &state.host_info.d_allowed),
                ]);

                out
            })
            .map(|v| Some(make_tree_element("disk_usage_summary", v))),
        )
    }

    pub fn project_attach(self) -> ElementFuture {
        let mut authenticator = String::new();
        let mut url = String::new();
        let mut project_name = String::new();
        let mut use_config_file = false;

        for child in &self.incoming.children {
            match &*child.name {
                "authenticator" => {
                    let _ = authenticator.unmarshal(child);
                }
                "url" => {
                    let _ = url.unmarshal(child);
                }
                "project_name" => {
                    let _ = project_name.unmarshal(child);
                }
                "use_config_file" => {
                    let _ = use_config_file.unmarshal(child);
                }
                _ => {}
            }
        }

        util::mutex_critical(self.state, move |state| {
            let project_init = state.project_init.take();
            let (url, authenticator) = match project_init.as_ref() {
                Some(project_init) => {
                    if project_init.url.is_empty() {
                        return Some(make_error("Missing URL"));
                    }

                    if project_init.account_key.is_empty() {
                        return Some(make_error("Missing authenticator"));
                    }

                    (project_init.url.clone(), project_init.account_key.clone())
                }
                None => {
                    if url.is_empty() {
                        return Some(make_error("Missing URL"));
                    }

                    if authenticator.is_empty() {
                        return Some(make_error("Missing authenticator"));
                    }
                    (url.clone(), authenticator.clone())
                }
            };

            for proj in &state.projects.data {
                if proj.master_url() == url {
                    return Some(make_error("Already attached to project"));
                }
            }

            state.project_attach = Default::default();
            *state.project_attach.error.lock().unwrap() = state
                .add_project(&url, &authenticator, &project_name, false)
                .err();

            project_init.map(|project_init| {
                let _ = project_init.remove().map_err(|err| {
                    state.messages.insert(
                        None,
                        common::MessagePriority::InternalError,
                        state.clock_source.now(),
                        &format!("Can't delete project init file: {}", err),
                    );
                });
            });

            Some(treexml::Element::new("success"))
        })
    }

    pub fn project_attach_poll(self) -> ElementFuture {
        let reply = await!(util::mutex_critical(self.state, |state| {
            let mut children = Vec::new();
            children.append(
                &mut state
                    .project_attach
                    .messages
                    .iter()
                    .map(|m| make_text_element("message", m))
                    .collect(),
            );
            children.push(make_text_element(
                "error_num",
                state
                    .project_attach
                    .error
                    .lock()
                    .unwrap()
                    .as_ref()
                    .map(|v| v.rpc_id())
                    .unwrap_or(0),
            ));

            Ok(children)
        }))?;

        Ok(Some(make_tree_element("project_attach_reply", reply)))
    }

    pub fn acct_mgr_info(self) -> ElementFuture {
        util::mutex_critical(self.state, |state| {
            let mut v = vec![
                make_text_element("acct_mgr_url", &state.acct_mgr_info.master_url),
                make_text_element("acct_mgr_name", &state.acct_mgr_info.project_name),
            ];

            if let Some(ref s) = state.acct_mgr_info.login_name {
                v.push(treexml::Element::new("have_credentials"));
            }

            if let acct_mgr::CookieStatus::Required(ref failure_url) =
                state.acct_mgr_info.cookie_status
            {
                v.push(treexml::Element::new("cookie_required"));
                v.push(make_text_element("cookie_failure_url", failure_url));
            }

            v
        })
        .map(|v| Some(make_tree_element("acct_mgr_info", v)))
    }

    pub fn get_cc_status(self) -> ElementFuture {
        util::mutex_critical(self.state, |state| {
            let now = state.clock_source.now();
            Ok(vec![
                make_text_element("ams_password_error", state.acct_mgr_info.password_error),
                make_text_element(
                    "task_suspend_reason",
                    state.suspend_reason.map(|v| v.into()).unwrap_or(0),
                ),
                make_text_element("task_mode", u8::from(state.run_mode.get_current())),
                make_text_element("task_mode_perm", u8::from(state.run_mode.get_perm())),
                make_text_element("task_mode_delay", state.run_mode.delay().num_seconds()),
                make_text_element(
                    "gpu_suspend_reason",
                    state.gpu_suspend_reason.map(|v| u8::from(v)).unwrap_or(0),
                ),
                make_text_element("gpu_mode", u8::from(state.gpu_run_mode.get_current())),
                make_text_element("gpu_mode_perm", u8::from(state.gpu_run_mode.get_perm())),
                make_text_element("gpu_mode_delay", state.gpu_run_mode.delay().num_seconds()),
                make_text_element("network_mode", 0),
                make_text_element("disallow_attach", state.cc_config.disallow_attach as u8),
                make_text_element("simple_gui_only", state.cc_config.simple_gui_only as u8),
                make_text_element("max_event_log_lines", state.cc_config.max_event_log_lines),
            ])
        })
        .map(|v| Some(make_tree_element("cc_status", v)))
    }

    pub fn get_statistics(self) -> ElementFuture {
        util::mutex_critical(self.state, |state| {
            Ok(state
                .projects
                .data
                .iter()
                .map(|p: &projects::Project| {
                    (p.master_url(), p.data.lock().unwrap().statistics.clone())
                })
                .collect())
        })
        .and_then(|stats| {
            stats
                .into_iter()
                .map(|data| {
                    let (master_url, stat_data) = data;
                    make_tree_element("project_statistics", {
                        let mut v = Vec::new();
                        v.push(make_text_element("master_url", master_url));
                        v.append(
                            &mut stat_data
                                .into_iter()
                                .map(|i: projects::DailyStats| -> treexml::Element {
                                    make_tree_element(
                                        "daily_statistics",
                                        vec![
                                            make_text_element("day", i.day),
                                            make_text_element(
                                                "user_total_credit",
                                                i.user_total_credit,
                                            ),
                                            make_text_element(
                                                "user_expavg_credit",
                                                i.user_expavg_credit,
                                            ),
                                            make_text_element(
                                                "host_total_credit",
                                                i.host_total_credit,
                                            ),
                                            make_text_element(
                                                "host_expavg_credit",
                                                i.host_expavg_credit,
                                            ),
                                        ],
                                    )
                                })
                                .collect(),
                        );
                        v
                    })
                })
                .collect()
        })
        .map(|stats_xml| Some(make_tree_element("statistics", stats_xml)))
    }
}
