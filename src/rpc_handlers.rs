extern crate std;

extern crate treexml;
extern crate treexml_util;

use common;
use constants;
use context;
use state;

use self::std::io::Read;
use self::treexml_util::Unmarshaller;

use common::ProjAm;

use self::treexml_util::{make_tree_element, make_text_element, make_cdata_element};

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
            self.context.await(
                move |state| state.unwrap().messages.len(),
            ),
        ))
    }

    pub fn get_messages(&self) -> Option<treexml::Element> {
        let seqno = treexml_util::find_value::<usize>("seqno", self.incoming)
            .ok()
            .unwrap_or(None);
        Some(self.context.await(move |state| {
            state.unwrap().messages.to_xml(seqno)
        }))
    }

    pub fn get_notices(&self) -> Option<treexml::Element> {
        Some(treexml::Element::new("notices"))
    }

    pub fn get_state(&self) -> Option<treexml::Element> {
        Some(self.context.await(move |state| {
            treexml::Element::from(state.unwrap())
        }))
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
                self.context.await_force(|state| {
                    let mut out = Vec::new();
                    for proj in &state.projects.data {
                        out.push({
                            treexml::Element {
                                name: "project".into(),
                                children: vec![
                                    make_text_element("master_url", proj.master_url()),
                                    make_text_element(
                                        "disk_usage",
                                        proj.data.await_force(|project| project.disk_usage)
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
                }),
            )
        })
    }

    pub fn project_attach(&self) -> Option<treexml::Element> {
        let mut authenticator = String::new();
        let mut url = String::new();
        let mut project_name = String::new();
        let mut use_config_file = false;

        for child in &self.incoming.children {
            authenticator.unmarshal("authenticator", &child);
            url.unmarshal("url", &child);
            project_name.unmarshal("project_name", &child);
            use_config_file.unmarshal("use_config_file", &child);
        }

        self.context.await_mut(move |s| {
            let mut state = s.unwrap();

            let (url, authenticator) = if use_config_file {
                if state.project_init.url.is_empty() {
                    return Some(make_error("Missing URL"));
                }

                if state.project_init.account_key.is_empty() {
                    return Some(make_error("Missing authenticator"));
                }

                (
                    state.project_init.url.clone(),
                    state.project_init.account_key.clone(),
                )
            } else {
                if url.is_empty() {
                    return Some(make_error("Missing URL"));
                }

                if authenticator.is_empty() {
                    return Some(make_error("Missing authenticator"));
                }
                (url.clone(), authenticator.clone())
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

            if url == state.project_init.url {
                state.project_init.remove().map_err(|err| {
                    state.messages.insert(
                        None,
                        common::MessagePriority::InternalError,
                        (state.clock_source)(),
                        &format!("Can't delete project init file: {}", err),
                    );
                });
            }

            Some(treexml::Element::new("success"))
        })
    }

    pub fn project_attach_poll(&self) -> Option<treexml::Element> {
        Some(make_tree_element(
            "project_attach_reply",
            self.context.await(|s| {
                let state = s.unwrap();
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
                        .map(|err| i64::from(err))
                        .unwrap_or(0),
                ));

                children
            }),
        ))
    }
}
