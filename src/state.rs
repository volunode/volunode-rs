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

pub type ContextData<V> = Arc<RwLock<Option<V>>>;

pub struct ContextMonad<V, T: Send + 'static> {
    pub f: Box<Fn() -> (ContextData<V>, T) + Send + 'static>,
}

impl<V: 'static, T: Send + 'static> ContextMonad<V, T> {
    pub fn bind<F, U>(self, func: F) -> ContextMonad<V, U>
    where
        F: Fn(Option<&V>, T) -> U,
        F: Send + 'static,
        U: Send,
    {
        ContextMonad::<V, U> {
            f: Box::new(move || {
                let (c, t) = (self.f)();
                let u = func(c.read().unwrap().as_ref(), t);
                (c, u)
            }),
        }
    }

    pub fn bind_mut<F, U>(self, func: F) -> ContextMonad<V, U>
    where
        F: Fn(Option<&mut V>, T) -> U,
        F: Send + 'static,
        U: Send,
    {
        ContextMonad::<V, U> {
            f: Box::new(move || {
                let (c, t) = (self.f)();
                let u = func(c.write().unwrap().as_mut(), t);
                (c, u)
            }),
        }
    }

    pub fn bind_rwlock<F, U>(self, func: F) -> ContextMonad<V, U>
    where
        F: Fn(&RwLock<Option<V>>, T) -> U,
        F: Send + 'static,
        U: Send,
    {
        ContextMonad::<V, U> {
            f: Box::new(move || {
                let (c, t) = (self.f)();
                let u = func(&c, t);
                (c, u)
            }),
        }
    }

    pub fn assemble(self) -> Box<Fn() -> T + Send + 'static> {
        Box::new(move || (self.f)().1)
    }

    pub fn run(self) -> std::thread::JoinHandle<T> {
        std::thread::spawn({
            move || self.assemble()()
        })
    }

    pub fn await(self) -> T {
        self.run().join().unwrap()
    }
}

pub struct Context<V: Send + Sync + 'static> {
    data: ContextData<V>,
}

impl<V: Send + Sync + 'static> Drop for Context<V> {
    fn drop(&mut self) {
        *self.data.write().unwrap() = None;
    }
}

impl<V: Send + Sync + 'static> Context<V> {
    pub fn new(v: V) -> Self {
        Self { data: Arc::new(RwLock::new(Some(v))) }
    }

    pub fn compose(&self) -> ContextMonad<V, ()> {
        ContextMonad {
            f: {
                Box::new({
                    let data = self.data.clone();
                    move || (data.clone(), ())
                })
            },
        }
    }

    pub fn run<F, T>(&self, f: F) -> std::thread::JoinHandle<T>
    where
        F: Fn(Option<&V>) -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        self.compose().bind(move |data, _| f(data)).run()
    }

    pub fn await<F, T>(&self, f: F) -> T
    where
        F: Fn(Option<&V>) -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        self.run(f).join().unwrap()
    }

    pub fn run_mut<F, T>(&self, f: F) -> std::thread::JoinHandle<T>
    where
        F: Fn(Option<&mut V>) -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        self.compose().bind_mut(move |data, _| f(data)).run()
    }

    pub fn await_mut<F, T>(&self, f: F) -> T
    where
        F: Fn(Option<&mut V>) -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        self.run_mut(f).join().unwrap()
    }
}

pub struct ClientState {
    pub messages: messages::SafeLogger,

    pub host_info: hostinfo::HostInfo,
    pub projects: projects::Projects,
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

impl ClientState {
    pub fn new(messages: messages::SafeLogger) -> Self {
        {
            let clock_source = Arc::new(move || std::time::SystemTime::now().into());
            let mut v = Self {
                messages: messages.clone(),
                host_info: hostinfo::HostInfo::default(),
                projects: projects::Projects::new(clock_source.clone(), messages.clone()),
            };

            v
        }
    }

    pub fn write_state_file(&self) -> Result<(), std::io::Error> {
        std::fs::File::create(constants::STATE_FILE_NAME)?
            .write_fmt(format_args!("{}", treexml::Element::from(self)))?;
        Ok(())
    }

    pub fn attach_project(&self, url: &str) {}
}
