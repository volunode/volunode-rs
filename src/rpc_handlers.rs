extern crate std;
extern crate futures;
extern crate treexml;
extern crate treexml_util;

use acct_mgr;
use common;
use constants;
use context;
use projects;
use state;

use self::std::io::Read;
use self::futures::Future;
use self::treexml_util::Unmarshaller;

use common::ProjAm;

use self::std::collections::*;
use self::treexml_util::{make_tree_element, make_text_element};

fn make_error(v: &str) -> treexml::Element {
    make_text_element("error", v)
}

/// Contains handlers to RPC calls
pub struct H<'a, 'b> {
    pub context: &'a context::Context<state::ClientState>,
    pub incoming: &'b treexml::Element,
}

impl<'a, 'b> H<'a, 'b> {
    pub fn get_message_count(&self) -> Option<treexml::Element> {
        Some(make_text_element(
            "seqno",
            self.context
                .run_force(move |state| state.messages.len())
                .wait()
                .unwrap(),
        ))
    }

    pub fn get_messages(&self) -> Option<treexml::Element> {
        let seqno = treexml_util::find_value::<usize>("seqno", self.incoming)
            .ok()
            .unwrap_or(None);
        Some(
            self.context
                .run_force(move |state| state.messages.to_xml(seqno))
                .wait()
                .unwrap(),
        )
    }

    pub fn get_notices(&self) -> Option<treexml::Element> {
        Some(treexml::Element::new("notices"))
    }

    pub fn get_state(&self) -> Option<treexml::Element> {
        Some(
            self.context
                .run_force(move |state| treexml::Element::from(state))
                .wait()
                .unwrap(),
        )
    }

    pub fn get_all_projects_list(&self) -> Option<treexml::Element> {
        match std::fs::File::open(constants::ALL_PROJECTS_LIST_FILENAME) {
            Err(_) => None,
            Ok(mut file) => {
                let mut s = String::new();
                match file.read_to_string(&mut s) {
                    Err(_) => None,
                    Ok(_) => {
                        match treexml::Document::parse(
                            std::io::Cursor::new(format!("<root>{}</root>", &s)),
                        ) {
                            Err(_) => None,
                            Ok(doc) => {
                                Some(make_tree_element("projects", doc.root.unwrap().children))
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn get_disk_usage(&self) -> Option<treexml::Element> {
        Some({
            make_tree_element(
                "disk_usage_summary",
                self.context
                    .run_force(|state| {
                        let mut out = Vec::new();
                        for proj in &state.projects.data {
                            out.push({
                                treexml::Element {
                                    name: "project".into(),
                                    children: vec![
                                        make_text_element(
                                            "master_url",
                                            proj.master_url()
                                        ),
                                        make_text_element(
                                            "disk_usage",
                                            proj.data.lock().unwrap().disk_usage
                                        ),
                                    ],
                                    ..Default::default()
                                }
                            });
                        }

                        out.append(&mut vec![
                            make_text_element(
                                "d_total",
                                &state.host_info.d_total
                            ),
                            make_text_element(
                                "d_free",
                                &state.host_info.d_free
                            ),
                        ]);

                        out
                    })
                    .wait()
                    .unwrap(),
            )
        })
    }

    pub fn project_attach(&self) -> Option<treexml::Element> {
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

        self.context
            .run_mut_force(move |state| {
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
            .wait()
            .unwrap()
    }

    pub fn project_attach_poll(&self) -> Option<treexml::Element> {
        Some(make_tree_element(
            "project_attach_reply",
            self.context
                .run_force(|state| {
                    let mut children = Vec::new();
                    children.append(&mut state
                        .project_attach
                        .messages
                        .iter()
                        .map(|m| make_text_element("message", m))
                        .collect());
                    children.push(make_text_element(
                        "error_num",
                        state
                            .project_attach
                            .error
                            .lock()
                            .unwrap()
                            .as_ref()
                            .map(i64::from)
                            .unwrap_or(0),
                    ));

                    children
                })
                .wait()
                .unwrap(),
        ))
    }

    pub fn acct_mgr_info(&self) -> Option<treexml::Element> {
        Some(make_tree_element(
            "acct_mgr_info",
            self.context
                .run_force(|state| {
                    let mut v =
                        vec![
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
                .wait()
                .unwrap(),
        ))
    }

    pub fn get_cc_status(&self) -> Option<treexml::Element> {
        Some(make_tree_element(
            "cc_status",
            self.context
                .run_force(|state: &state::ClientState| {
                    let now = state.clock_source.now();
                    vec![
                        make_text_element("ams_password_error", state.acct_mgr_info.password_error),
                        make_text_element("task_suspend_reason", state.suspend_reason.map(|v| v.into()).unwrap_or(0)),
                        make_text_element("task_mode", u8::from(state.run_mode.get_current())),
                        make_text_element("task_mode_perm", u8::from(state.run_mode.get_perm())),
                        make_text_element("task_mode_delay", state.run_mode.delay().num_seconds()),
                        make_text_element("gpu_suspend_reason", state.gpu_suspend_reason.map(|v| u8::from(v)).unwrap_or(0)),
                        make_text_element("gpu_mode", u8::from(state.gpu_run_mode.get_current())),
                        make_text_element("gpu_mode_perm", u8::from(state.gpu_run_mode.get_perm())),
                        make_text_element("gpu_mode_delay", state.gpu_run_mode.delay().num_seconds()),
                        make_text_element("network_mode", 0),
                        make_text_element("disallow_attach", state.cc_config.disallow_attach as u8),
                        make_text_element("simple_gui_only", state.cc_config.simple_gui_only as u8),
                        make_text_element("max_event_log_lines", state.cc_config.max_event_log_lines),
                    ]
                })
                .wait()
                .unwrap(),
        ))
    }

    pub fn get_statistics(&self) -> Option<treexml::Element> {
        Some(make_tree_element(
            "statistics",
            {
                let stats: HashMap<String, Vec<projects::DailyStats>> = self.context.run_force(|state: &state::ClientState| {
                    state.projects.data.iter().map(|p: &projects::Project| (p.master_url(), p.data.lock().unwrap().statistics.clone())).collect()
                })
                    .wait()
                    .unwrap();

                stats.into_iter().map(|data| {
                    let (master_url, stat_data) = data;
                    make_tree_element("project_statistics",
                    {
                        let mut v = Vec::new();
                        v.push(make_text_element("master_url", master_url));
                        v.append(&mut stat_data.into_iter().map(|i: projects::DailyStats| -> treexml::Element {
                            make_tree_element("daily_statistics", vec![
                                make_text_element("day", i.day),
                                make_text_element("user_total_credit", i.user_total_credit),
                                make_text_element("user_expavg_credit", i.user_expavg_credit),
                                make_text_element("host_total_credit", i.host_total_credit),
                                make_text_element("host_expavg_credit", i.host_expavg_credit),
                            ])
                        }).collect());
                        v
                    })
                }).collect()
            },

        ))
    }
}
