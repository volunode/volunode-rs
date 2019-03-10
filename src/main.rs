#![recursion_limit = "1024"]

mod acct_mgr;
mod acct_setup;
mod app;
mod cc_config;
mod cert_sig;
mod common;
mod constants;
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

use process::Process;
use rpc::RPCServer;
use state::ClientState;

use futures::Future;
use std::sync::{Arc, Mutex};
use tokio_core::reactor::{Core, Handle};

struct Daemon {
    state: ClientState,
    rpc_server: Option<RPCServer>,
}

impl Daemon {
    pub fn new(rpc_settings: Option<rpc::RPCSettings>) -> Self {
        let state = state::ClientState::new(Arc::new(messages::StandardLogger::default()));

        let rpc_server =
            rpc_settings.map(|settings| rpc::RPCServer::new(Arc::clone(&state), settings));

        Self { state, rpc_server }
    }
}

fn main() {
    let addr = std::env::var(constants::ENV_RPC_ADDR)
        .ok()
        .map(|v| v.parse().unwrap());
    let password = std::env::var(constants::ENV_RPC_PASSWORD).ok();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let daemon = Daemon::new(addr.map(|addr| rpc::RPCSettings { addr, password }));

    handle.spawn(
        util::mutex_critical(Arc::clone(&daemon.state), |state| {
            state.messages.insert(
                None,
                common::MessagePriority::Info,
                std::time::SystemTime::now().into(),
                "Main thread is up and parked",
            );
            Ok(())
        })
        .or_else(|_| Err(())),
    );

    loop {
        std::thread::park();
    }
}
