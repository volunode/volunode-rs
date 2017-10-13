#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde;

extern crate tokio_proto;

mod acct_setup;
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

use context::Context;
use process::Process;
use rpc::RPCServer;
use state::ClientState;

use std::sync::Arc;

fn launch_service_threads(context: &context::Context<state::ClientState>) {
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
        .run();

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
        .run();

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
        .run();
}

struct Daemon {
    context: Arc<Context<ClientState>>,
    rpc_server: RPCServer,
}

impl Daemon {
    pub fn run(addr: std::net::SocketAddr, password: Option<String>) -> Self {
        let context = Arc::new(context::Context::new(state::ClientState::new(
            Arc::new(messages::StandardLogger::default()),
        )));

        let srv = rpc::start_rpc_server(Arc::clone(&context), addr, password);

        Self {
            context: context,
            rpc_server: srv,
        }
    }
}

fn main() {
    let addr = format!(
        "127.0.0.1:{}",
        std::env::var(constants::ENV_RPC_PORT)
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(constants::DEFAULT_RPC_PORT)
    ).parse()
        .unwrap();
    let password = std::env::var(constants::ENV_RPC_PASSWORD).ok();

    let daemon = Daemon::run(addr, password);

    launch_service_threads(&daemon.context);

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
