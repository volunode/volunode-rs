#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde;

mod acct_setup;
mod app;
mod cc_config;
mod common;
mod constants;
mod coproc;
mod errors;
mod file_names;
mod hostinfo;
mod messages;
mod process;
mod project_init;
mod projects;
mod rpc;
mod state;
mod util;
mod workunit;

use std::sync::Arc;
use std::str;

fn launch_service_threads(context: &state::Context<state::ClientState>) {
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

fn main() {
    let context = Arc::new(state::Context::new(state::ClientState::new(
        Arc::new(messages::StandardLogger::default()),
    )));
    let addr = format!(
        "127.0.0.1:{}",
        std::env::var(constants::ENV_RPC_PORT)
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
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

    launch_service_threads(&context);

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
