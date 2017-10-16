#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde;
extern crate futures;

extern crate tokio_proto;

mod acct_setup;
mod acct_mgr;
mod app;
mod cc_config;
mod cert_sig;
mod common;
mod constants;
mod context;
mod coproc;
mod errors;
mod file_info;
mod file_names;
mod hostinfo;
mod messages;
mod process;
mod project_init;
mod projects;
mod rpc;
mod rpc_handlers;
mod state;
mod tasks;
mod util;
mod workunit;

use context::{Context, ContextFuture};
use process::Process;
use rpc::RPCServer;
use state::ClientState;

use std::sync::Arc;

fn launch_service_threads(
    context: &context::Context<state::ClientState>,
) -> Vec<ContextFuture<()>> {
    vec![
        context
            .compose()
            .bind_rwlock(|r, _| loop {
                match r.read().unwrap().as_ref() {
                    Some(state) => {
                        state.messages.insert(
                            None,
                            common::MessagePriority::Info,
                            std::time::SystemTime::now().into(),
                            "Service 1 ping",
                        )
                    }
                    None => {
                        return;
                    }
                };
                std::thread::sleep(std::time::Duration::from_millis(2500));
            })
            .run(),

        context
            .compose()
            .bind_rwlock(|r, _| loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
                match r.read().unwrap().as_ref() {
                    Some(state) => {
                        state.messages.insert(
                            None,
                            common::MessagePriority::Info,
                            std::time::SystemTime::now().into(),
                            "Service 2 ping",
                        )
                    }
                    None => {
                        return;
                    }
                };
                std::thread::sleep(std::time::Duration::from_millis(500));
            })
            .run(),

        context
            .compose()
            .bind_rwlock(|r, _| loop {
                match r.write().unwrap().as_mut() {
                    Some(ref mut state) => {
                        state.messages.insert(
                            None,
                            common::MessagePriority::Info,
                            std::time::SystemTime::now().into(),
                            "Mutating service ping",
                        )
                    }
                    None => {
                        return;
                    }
                };
                std::thread::sleep(std::time::Duration::from_millis(2000));
            })
            .run(),
    ]
}

struct RPCSettings {
    pub addr: std::net::SocketAddr,
    pub password: Option<String>,
}

enum RPCEnabled {
    Yes(RPCSettings),
    No,
}

impl From<(Option<std::net::SocketAddr>, Option<String>)> for RPCEnabled {
    fn from(v: (Option<std::net::SocketAddr>, Option<String>)) -> Self {
        let (addr, password) = v;
        match addr {
            Some(v) => RPCEnabled::Yes(RPCSettings {
                addr: v,
                password: password,
            }),
            None => RPCEnabled::No,

        }
    }
}

struct Daemon {
    context: Arc<Context<ClientState>>,
    rpc_server: Option<Arc<RPCServer>>,
    service_threads: Vec<ContextFuture<()>>,
}

impl Daemon {
    pub fn run(rpc_enable: RPCEnabled) -> Self {
        let context = Arc::new(context::Context::new(state::ClientState::new(
            Arc::new(messages::StandardLogger::default()),
        )));

        let srv = match rpc_enable {
            RPCEnabled::Yes(settings) => Some(rpc::RPCServer::run(
                Arc::clone(&context),
                settings.addr,
                settings.password,
            )),
            RPCEnabled::No => None,
        };

        Self {
            service_threads: Default::default(), //launch_service_threads(&*context),
            context: context,
            rpc_server: srv,
        }
    }
}

fn main() {
    let addr = std::env::var(constants::ENV_RPC_ADDR).ok().map(|v| {
        v.parse().unwrap()
    });
    let password = std::env::var(constants::ENV_RPC_PASSWORD).ok();


    let daemon = Daemon::run((addr, password).into());

    daemon.context.run(|state| {
        state.unwrap().messages.insert(
            None,
            common::MessagePriority::Info,
            std::time::SystemTime::now().into(),
            "Main thread is up and parked",
        );
    });

    loop {
        std::thread::park();
    }
}
