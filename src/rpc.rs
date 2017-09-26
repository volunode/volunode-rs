extern crate std;

extern crate bytes;
extern crate chan;
extern crate crypto;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;
extern crate treexml;
extern crate treexml_util;

use acct_setup;
use common;
use constants;
use context;
use messages;
use rpc_handlers as handlers;
use state;

use self::futures::{future, Future, BoxFuture};
use self::tokio_io::{AsyncRead, AsyncWrite};
use self::tokio_io::codec::{Decoder, Encoder, Framed};
use self::tokio_service::Service;
use self::treexml_util::Unmarshaller;
use self::bytes::BytesMut;
use self::crypto::digest::Digest;
use self::crypto::md5::Md5;
use self::io::Read;

use common::ProjAm;

use std::io;
use std::sync::{Arc, RwLock};
use self::treexml_util::{make_tree_element, make_text_element, make_cdata_element};

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
                    Ok(s) => {
                        match s.root {
                            Some(r) => Ok(Some(r)),
                            None => Err(parse_err()),
                        }
                    }
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

#[derive(Clone)]
struct RpcService {
    cpu_pool: futures_cpupool::CpuPool,

    context: Arc<context::Context<state::ClientState>>,
    conn_status: Arc<RwLock<AuthState>>,

    rpcpass: Option<String>,
}

impl RpcService {
    fn new(
        c: Arc<context::Context<state::ClientState>>,
        cpu_pool: futures_cpupool::CpuPool,
        pass: Option<String>,
    ) -> RpcService {
        RpcService {
            cpu_pool: cpu_pool,
            context: c,
            conn_status: Arc::new(RwLock::new(AuthState::New)),
            rpcpass: pass,
        }
    }

    fn salted_hash(&self, nonce: &str) -> Option<String> {
        self.rpcpass.as_ref().map(|v| {
            let input = format!("{}{}", nonce, v);
            let mut hasher = Md5::new();
            hasher.input_str(&input);
            hasher.result_str()
        })
    }

    fn process_request(&self, v: &treexml::Element) -> Option<treexml::Element> {
        let h = H {
            context: &*self.context,
            incoming: v,
        };

        (match &*v.name {
             "get_message_count" => H::get_message_count,
             "get_messages" => H::get_messages,
             "get_notices" => H::get_notices,
             "get_state" => H::get_state,
             "get_all_projects_list" => H::get_all_projects_list,
             "get_disk_usage" => H::get_disk_usage,
             "project_attach" => H::project_attach,
             "project_attach_poll" => H::project_attach_poll,
             _ => {
                 return None;
             }
         })(&h)
    }
}

impl Service for RpcService {
    type Request = treexml::Element;
    type Response = treexml::Element;

    type Error = io::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn call(&self, full_request: Self::Request) -> Self::Future {
        let rsp_ok = |v: Option<treexml::Element>| {
            future::ok(make_tree_element(
                "boinc_gui_rpc_reply",
                match v {
                    Some(v) => vec![v.into()],
                    None => vec![],
                },
            )).boxed()
        };
        let rsp_err = |v| future::err(v).boxed();

        let unauthorize = |s: &mut AuthState| {
            *s = AuthState::Unauthorized;
            rsp_ok(Some(treexml::Element::new("unauthorized")))
        };
        let authorize = |s: &mut AuthState| {
            *s = AuthState::Ready;
            rsp_ok(Some(treexml::Element::new("authorized")))
        };

        let mut s = self.conn_status.write().unwrap();
        let current_status = (*s).clone();
        if full_request.name == "boinc_gui_rpc_request" && full_request.children.len() > 0 {
            let req = full_request.children.get(0).unwrap();
            match current_status {
                AuthState::New => {
                    if req.name == "auth1" {
                        match &self.rpcpass {
                            &Some(_) => {
                                let nonce = format!(
                                    "{}",
                                    std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs()
                                );
                                *s = AuthState::ChallengeSent(nonce.clone());
                                let v = make_text_element("nonce", &nonce);
                                println!(
                                    "Salted hash must be {}",
                                    self.salted_hash(&nonce).unwrap()
                                );

                                rsp_ok(Some(v))
                            }
                            &None => authorize(&mut *s),
                        }
                    } else {
                        unauthorize(&mut *s)
                    }
                }
                AuthState::ChallengeSent(ref nonce) => {
                    match treexml_util::find_value::<String>("nonce_hash", &req) {
                        Ok(opt_response) => {
                            match opt_response {
                                Some(response) => {
                                    if self.salted_hash(nonce).unwrap() == response {
                                        authorize(&mut *s)
                                    } else {
                                        unauthorize(&mut *s)
                                    }
                                }
                                None => unauthorize(&mut *s),
                            }
                        }
                        Err(_) => unauthorize(&mut *s),
                    }
                }
                AuthState::Unauthorized => {
                    rsp_err(io::Error::new(
                        io::ErrorKind::Other,
                        "Still banging into closed door",
                    ))
                }
                AuthState::Ready => rsp_ok(self.process_request(&req).into()),
            }
        } else {
            rsp_err(io::Error::new(io::ErrorKind::Other, "Invalid request"))
        }
    }
}

pub fn start_rpc_server(
    context: Arc<context::Context<state::ClientState>>,
    addr: std::net::SocketAddr,
    password: Option<String>,
) -> () {
    let server = tokio_proto::TcpServer::new(RPCProto, addr);
    let thread_pool = futures_cpupool::CpuPool::new(10);
    context.run({
        let addr = addr.clone();
        move |state| {
            state.unwrap().messages.insert(
                None,
                common::MessagePriority::Debug,
                std::time::SystemTime::now().into(),
                &format!("Starting RPC server at {}", &addr),
            );
        }
    });
    server.with_handle({
        let context = context.clone();
        let password = password.clone();
        let thread_pool = thread_pool.clone();
        move |_| {
            let context = context.clone();
            let password = password.clone();
            let thread_pool = thread_pool.clone();
            move || {
                Ok(RpcService::new(
                    context.clone(),
                    thread_pool.clone(),
                    password.clone(),
                ))
            }
        }
    });
}
