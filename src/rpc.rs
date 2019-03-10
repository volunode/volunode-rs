extern crate std;

extern crate bytes;
extern crate chan;
extern crate crypto;
extern crate failure;
extern crate futures_await as futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;
extern crate treexml;
extern crate treexml_util;

use common;
use errors;
use rpc_handlers as handlers;
use state;
use util;

use self::bytes::BytesMut;
use self::crypto::digest::Digest;
use self::crypto::md5::Md5;
use self::futures::future::{err, ok, Executor};
use self::futures::prelude::*;
use self::futures::{future, BoxFuture};
use self::tokio_core::reactor::{Core, Handle};
use self::tokio_io::codec::{Decoder, Encoder, Framed};
use self::tokio_io::{AsyncRead, AsyncWrite};
use self::tokio_service::Service;

use self::treexml_util::{make_text_element, make_tree_element};
use std::io;
use std::sync::{Arc, Mutex, RwLock};

use self::handlers::H;

pub struct RPCCodec;

impl RPCCodec {
    fn terminator(&self) -> u8 {
        3
        // b'\n' // this is for per-line manual debugging
    }
}

impl Decoder for RPCCodec {
    type Item = treexml::Element;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let parse_err = || io::Error::new(io::ErrorKind::Other, "Failed to parse document");

        match buf.iter().position(|&b| b == self.terminator()) {
            Some(i) => {
                let line = buf.split_to(i);
                buf.split_to(1);

                println!("{}", String::from_utf8_lossy(line.as_ref()));

                match treexml::Document::parse(line.as_ref()) {
                    Ok(s) => match s.root {
                        Some(r) => Ok(Some(r)),
                        None => Err(parse_err()),
                    },
                    Err(_) => Err(parse_err()),
                }
            }
            None => Ok(None),
        }
    }
}

impl Encoder for RPCCodec {
    type Item = treexml::Element;
    type Error = io::Error;

    fn encode(&mut self, v: Self::Item, buf: &mut BytesMut) -> Result<(), Self::Error> {
        let s = format!("{}", v);
        println!("{}", &s);
        buf.extend_from_slice(s.as_bytes());
        buf.extend_from_slice(&[self.terminator()]);
        Ok(())
    }
}

pub struct RPCProto;

impl<T: AsyncRead + AsyncWrite + 'static> tokio_proto::pipeline::ServerProto<T> for RPCProto {
    type Request = treexml::Element;
    type Response = treexml::Element;

    type Transport = Framed<T, RPCCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(RPCCodec))
    }
}

#[derive(Clone, Debug)]
enum AuthState {
    New,
    ChallengeSent(String),
    Unauthorized,
    Ready,
}

impl Default for AuthState {
    fn default() -> Self {
        AuthState::New
    }
}

#[derive(Clone)]
struct RpcService {
    executor: Handle,

    state: Arc<Mutex<state::ClientState>>,
    conn_status: Arc<RwLock<AuthState>>,

    rpc_pass: Option<String>,
}

impl RpcService {
    fn new(
        executor: Handle,
        state: Arc<Mutex<state::ClientState>>,
        rpc_pass: Option<String>,
    ) -> RpcService {
        RpcService {
            executor,
            state,
            conn_status: Default::default(),
            rpc_pass,
        }
    }
}

fn process_rpc_request(
    state: Arc<Mutex<state::ClientState>>,
    incoming: treexml::Element,
) -> impl Future<Item = Option<treexml::Element>, Error = errors::Error> {
    (match &*incoming.name {
        "acct_mgr_info" => H::acct_mgr_info,
        "get_cc_status" => H::get_cc_status,
        "get_message_count" => H::get_message_count,
        "get_messages" => H::get_messages,
        "get_notices" => H::get_notices,
        "get_state" => H::get_state,
        "get_statistics" => H::get_statistics,
        "get_all_projects_list" => H::get_all_projects_list,
        "get_disk_usage" => H::get_disk_usage,
        "project_attach" => H::project_attach,
        "project_attach_poll" => H::project_attach_poll,
        _ => {
            return ok(None);
        }
    })(H { state, incoming })
}

fn rsp_ok(v: Option<treexml::Element>) -> treexml::Element {
    make_tree_element(
        "boinc_gui_rpc_reply",
        match v {
            Some(v) => vec![v.into()],
            None => vec![],
        },
    )
}

fn reject(s: &mut AuthState) -> treexml::Element {
    *s = AuthState::Unauthorized;
    rsp_ok(Some(treexml::Element::new("unauthorized")))
}

fn authorize(s: &mut AuthState) -> treexml::Element {
    *s = AuthState::Ready;
    rsp_ok(Some(treexml::Element::new("authorized")))
}

fn get_authorization_hash(rpc_pass: &str, nonce: &str) -> String {
    let input = format!("{}{}", nonce, rpc_pass);
    let mut hasher = Md5::new();
    hasher.input_str(&input);
    hasher.result_str()
}

#[async(boxed)]
fn rpc_service_call(
    state: Arc<Mutex<state::ClientState>>,
    auth_status: Arc<Mutex<AuthState>>,
    rpc_pass: Option<String>,
    full_request: treexml::Element,
) -> Result<treexml::Element, errors::Error> {
    let mut s = auth_status.lock().unwrap();
    let current_status = (*s).clone();
    if full_request.name == "boinc_gui_rpc_request" {
        if let Some(req) = full_request.children.get(0) {
            match current_status {
                AuthState::New => Ok(if req.name == "auth1" {
                    match rpc_pass {
                        Some(pass) => {
                            let nonce = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                                .to_string();
                            let hash = get_authorization_hash(&pass, &nonce);
                            println!("Salted hash must be {}", &hash);
                            *s = AuthState::ChallengeSent(hash);
                            let v = make_text_element("nonce", &nonce);

                            rsp_ok(Some(v))
                        }
                        None => authorize(&mut *s),
                    }
                } else {
                    reject(&mut *s)
                }),
                AuthState::ChallengeSent(ref expected_hash) => Ok(
                    match treexml_util::find_value::<String>("nonce_hash", req) {
                        Ok(opt_response) => match opt_response {
                            Some(client_hash) => {
                                if *expected_hash == client_hash {
                                    authorize(&mut *s)
                                } else {
                                    reject(&mut *s)
                                }
                            }
                            None => reject(&mut *s),
                        },
                        Err(_) => reject(&mut *s),
                    },
                ),
                AuthState::Unauthorized => Err(errors::Error::AuthError {
                    what: "Still banging into closed door".into(),
                }),
                AuthState::Ready => {
                    await!(process_rpc_request(Arc::clone(&state), req.clone()).map(rsp_ok))
                }
            }
        } else {
            Err(errors::Error::AuthError {
                what: "Request's root has no contents".into(),
            })
        }
    } else {
        Err(errors::Error::AuthError {
            what: "Request is empty".into(),
        })
    }
}

impl Service for RpcService {
    type Request = treexml::Element;
    type Response = treexml::Element;

    type Error = errors::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn call(&self, full_request: Self::Request) -> Self::Future {
        rpc_service_call(
            Arc::clone(&self.state),
            Arc::clone(&self.auth_status),
            self.rpc_pass.clone(),
            full_request,
        )
    }
}

pub struct RPCSettings {
    pub addr: std::net::SocketAddr,
    pub password: Option<String>,
}

pub struct RPCServer {
    state: Arc<Mutex<state::ClientState>>,
    settings: RPCSettings,
}

impl RPCServer {
    pub fn new(state: Arc<Mutex<state::ClientState>>, settings: RPCSettings) -> Self {
        Self { state, settings }
    }

    pub fn run(&self, core: &mut Core) -> Result<(), errors::Error> {
        let addr = self.settings.addr.clone();
        let password = self.settings.password.clone();
        core.run(util::mutex_critical(
            Arc::clone(&self.state),
            move |state| {
                state.messages.insert(
                    None,
                    common::MessagePriority::Debug,
                    std::time::SystemTime::now().into(),
                    &format!("Starting RPC server at {}", &addr),
                );
            },
        ))?;

        let server = tokio_proto::TcpServer::new(RPCProto, addr);
        server.with_handle(move |h| {
            let state = Arc::clone(&self.state);
            let password = password.clone();
            move || {
                Ok(RpcService::new(
                    h.clone(),
                    Arc::clone(&state),
                    password.clone(),
                ))
            }
        })?
    }
}
