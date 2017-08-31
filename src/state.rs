extern crate chan;
extern crate std;
extern crate treexml;

use common;
use constants;
use errors;
use hostinfo;
use messages;
use projects;

use std::io::Write;
use std::ops::Deref;

use std::sync::{Arc, RwLock};

pub struct ClientState {
    pub messages: messages::SafeLogger,

    pub host_info: Arc<RwLock<hostinfo::HostInfo>>,
    pub projects: Arc<RwLock<projects::Projects>>,
}

impl<'a> From<&'a ClientState> for treexml::Element {
    fn from(v: &ClientState) -> treexml::Element {
        let host_info = v.host_info.read().unwrap();
        let projects = v.projects.read().unwrap();

        treexml::ElementBuilder::new("client_state")
            .children(vec![&mut host_info.deref().into()])
            .children(
                projects
                    .deref()
                    .data
                    .iter()
                    .map(|v| treexml::Element::from(v).into())
                    .collect::<Vec<treexml::ElementBuilder>>()
                    .iter_mut()
                    .collect(),
            )
            .element()
    }
}

impl ClientState {
    pub fn new(messages: messages::SafeLogger) -> ClientState {
        {
            let clock_source = Arc::new(move || std::time::SystemTime::now().into());
            let mut v = ClientState {
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

    pub fn write_state_file(&self) -> Result<(), std::io::Error> {
        std::fs::File::create(constants::STATE_FILE_NAME)?
            .write_fmt(format_args!("{}", treexml::Element::from(self)))?;
        Ok(())
    }
}

pub type ClientStateAction = Arc<Fn(&RwLock<ClientState>) -> bool + Send + Sync>;

pub struct ClientStateReactor {
    data: Arc<RwLock<ClientState>>,
    tasks: chan::Receiver<ClientStateAction>,
    feeder: chan::Sender<ClientStateAction>,
}

impl ClientStateReactor {
    pub fn new(messages: messages::SafeLogger) -> ClientStateReactor {
        let (tx, rx) = chan::async();
        ClientStateReactor {
            data: Arc::new(RwLock::new(ClientState::new(messages))),
            tasks: rx,
            feeder: tx,
        }
    }

    pub fn queue<F>(&self, v: F)
    where
        F: 'static + Fn(&RwLock<ClientState>) -> bool + Send + Sync,
    {
        self.feeder.send(Arc::new(v));
    }

    pub fn oneshot<F>(&self, f: F)
    where
        F: 'static + Fn(&RwLock<ClientState>) + Send + Sync,
    {
        self.queue(move |cs| {
            f(cs);
            false
        })
    }

    pub fn await<T, F>(&self, f: F) -> T
    where
        T: 'static + Send,
        F: 'static + Fn(&RwLock<ClientState>) -> T + Send + Sync,
    {
        let (tx, rx) = chan::async();
        self.oneshot(move |cs| { tx.send(f(cs)); });
        rx.recv().unwrap()
    }

    pub fn run(&self, multithreaded: bool) {
        loop {
            match self.tasks.recv() {
                Some(f) => {
                    if multithreaded {
                        std::thread::spawn({
                            let data = self.data.clone();
                            let feeder = self.feeder.clone();
                            move || if f(&*data) {
                                feeder.send(f);
                            }
                        });
                    } else {
                        if f(&*self.data) {
                            self.feeder.send(f);
                        }
                    }
                }
                None => return,
            }
        }
    }
}
