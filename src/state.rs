extern crate treexml;

use common;
use errors;
use hostinfo;
use messages;
use projects;

use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};

pub struct ClientState {
    pub now: Arc<RwLock<Option<common::Time>>>,
    pub messages: messages::SafeLogger,

    pub host_info: Arc<RwLock<hostinfo::HostInfo>>,
    pub projects: Arc<RwLock<projects::Projects>>,
}

impl<'a> From<&'a ClientState> for treexml::Element {
    fn from(v: &ClientState) -> treexml::Element {
        let now = v.now.read().unwrap();
        let host_info = v.host_info.read().unwrap();

        treexml::ElementBuilder::new("client_state")
            .children(vec![&mut host_info.deref().into()])
            .element()
    }
}

impl ClientState {
    pub fn new(messages: messages::SafeLogger) -> ClientState {
        {
            let now = Arc::new(RwLock::new(None));
            let clock_source = Arc::new(move || now.read().unwrap().clone().unwrap());
            let mut v = ClientState {
                now: Arc::new(RwLock::new(None)),
                messages: messages.clone(),
                host_info: Arc::new(RwLock::new(hostinfo::HostInfo::default())),
                projects: Arc::new(RwLock::new(projects::Projects::new(
                    clock_source.clone(),
                    messages.clone(),
                ))),
            };

            v
        }
    }

    pub fn write_state_file(&self) -> Result<(), errors::Error> {
        Ok(())
    }
}
