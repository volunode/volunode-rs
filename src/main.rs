mod app;
mod common;
mod constants;
mod errors;
mod hostinfo;
mod messages;
mod projects;
mod rpc;
mod state;
mod util;
mod workunit;

use std::sync::{Arc, RwLock};
use std::str;

fn main() {
    let m: messages::SafeLogger = Arc::new(messages::StandardLogger::default());
    let reactor = Arc::new(state::ClientStateReactor::new(m.clone()));
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
        let reactor = reactor.clone();
        move || rpc::StartRpcServer(reactor, addr, password)
    });

    reactor.oneshot(|cs| {
        cs.read().unwrap().messages.insert(
            None,
            common::MessagePriority::Info,
            std::time::SystemTime::now().into(),
            "Main thread is up and parked",
        );
    });

    reactor.run(true);
}
