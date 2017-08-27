mod app;
mod common;
mod errors;
mod hostinfo;
mod messages;
mod projects;
mod rpc;
mod state;
mod util;
mod workunit;

use std::borrow::Borrow;
use std::sync::{Arc, RwLock};
use std::str;

fn main() {
    let m: messages::SafeLogger = Arc::new(messages::StandardLogger::default());
    let client_state = Arc::new(RwLock::new(state::ClientState::new(m.clone())));
    let addr = "127.0.0.1:31417".parse().unwrap();
    let password = Some("mypass".into());

    std::thread::spawn({
        let s = client_state.clone();
        move || rpc::StartRpcServer(s, addr, password)
    });

    let msgs: &(messages::Logger + Send + Sync) = m.borrow();
    msgs.insert(
        None,
        common::MessagePriority::Info,
        std::time::SystemTime::now().into(),
        "Main thread is up and parked",
    );

    loop {
        std::thread::park();
    }
}
