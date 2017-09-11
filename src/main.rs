#[macro_use]
extern crate serde;

mod app;
mod common;
mod constants;
mod errors;
mod hostinfo;
mod messages;
mod process;
mod projects;
mod rpc;
mod state;
mod util;
mod workunit;

use std::sync::{Arc, RwLock};
use std::str;

fn main() {
    let context = Arc::new(state::Context::new(state::ClientState::new(
        Arc::new(messages::StandardLogger::default()),
    )));
    let addr = format!(
        "127.0.0.1:{}",
        std::env::var(constants::ENV_RPC_PORT)
            .ok()
            .map(|p| p.parse::<u16>().ok())
            .unwrap_or(Some(constants::DEFAULT_RPC_PORT))
            .unwrap_or(constants::DEFAULT_RPC_PORT)
    ).parse()
        .unwrap();
    let password = Some("mypass".into());

    std::thread::spawn({
        let context = context.clone();
        move || rpc::start_rpc_server(context, addr, password)
    });

    let p = std::env::var("TEST_IPC").map(|path| {
        process::Process::new(&path, "./boinc_mmap_file", {
            let context = context.clone();
            let path = path.clone();
            move |msg| {
                context.run({
                    let path = path.clone();
                    move |state| {
                        state.unwrap().messages.insert(
                            None,
                            common::MessagePriority::Info,
                            std::time::SystemTime::now().into(),
                            &format!(
                                "Received message.\nE: {}\nV: {}",
                                path,
                                msg
                            ),
                        );
                    }
                });
            }
        })
    });

    context.run(|state| {
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
