extern crate bytes;
extern crate chan;
extern crate crypto;
extern crate futures;
extern crate futures_cpupool;
extern crate std;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;
extern crate treexml;

use self::futures::{future, Future, BoxFuture};
use self::tokio_io::{AsyncRead, AsyncWrite};
use self::tokio_io::codec::{Decoder, Encoder, Framed};
use self::tokio_service::Service;
use self::bytes::BytesMut;
use self::crypto::digest::Digest;
use self::crypto::md5::Md5;
use self::io::Read;

use std::io;
use std::sync::{Arc, RwLock};

use common;
use constants;
use messages;
use state;

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

    context: Arc<state::Context<state::ClientState>>,
    conn_status: Arc<RwLock<AuthState>>,

    rpcpass: Option<String>,
}

impl RpcService {
    fn new(
        c: Arc<state::Context<state::ClientState>>,
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
        match &*v.name {
            "get_message_count" => {
                Some(
                    treexml::ElementBuilder::new("seqno")
                        .text(format!(
                            "{}",
                            self.context.await(
                                move |state| state.unwrap().messages.len(),
                            )
                        ))
                        .element(),
                )
            }
            "get_messages" => {
                let seqno = v.find_value("seqno").unwrap_or(None);
                Some(
                    self.context
                        .await(move |state| state.unwrap().messages.to_xml(seqno))
                        .element(),
                )
            }
            "get_notices" => Some(treexml::Element::new("notices")),
            "get_state" => {
                Some(self.context.await(move |state| {
                    treexml::Element::from(state.unwrap())
                }))
            }
            "get_all_projects_list" => {
                match std::fs::File::open(constants::ALL_PROJECTS_LIST_FILENAME) {
                    Err(_) => None,
                    Ok(mut file) => {
                        let mut s = String::new();
                        match file.read_to_string(&mut s) {
                            Err(_) => None,
                            Ok(_) => {
                                match treexml::Document::parse(
                                    std::io::Cursor::new(format!("<root>{}</root>", &s)),
                                ) {
                                    Err(_) => None,
                                    Ok(doc) => {
                                        let mut e = treexml::Element::new("projects");
                                        e.children = doc.root.unwrap().children;
                                        Some(e)
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => None,
        }
    }
}

impl Service for RpcService {
    type Request = treexml::Element;
    type Response = treexml::Element;

    type Error = io::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn call(&self, full_request: Self::Request) -> Self::Future {
        let rsp_ok = |v: Option<treexml::Element>| {
            future::ok(
                treexml::ElementBuilder::new("boinc_gui_rpc_reply")
                    .children(
                        match v {
                            Some(v) => vec![v.into()],
                            None => vec![],
                        }.iter_mut()
                            .collect(),
                    )
                    .element(),
            ).boxed()
        };
        let rsp_err = |v| future::err(v).boxed();

        let unauthorize = |s: &mut AuthState| {
            *s = AuthState::Unauthorized;
            rsp_ok(Some(treexml::ElementBuilder::new("unauthorized").element()))
        };
        let authorize = |s: &mut AuthState| {
            *s = AuthState::Ready;
            rsp_ok(Some(treexml::ElementBuilder::new("authorized").element()))
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
                                let mut v = treexml::ElementBuilder::new("nonce");
                                v.text(nonce.clone());
                                println!(
                                    "Salted hash must be {}",
                                    self.salted_hash(&nonce).unwrap()
                                );

                                rsp_ok(Some(v.element()))
                            }
                            &None => authorize(&mut *s),
                        }
                    } else {
                        unauthorize(&mut *s)
                    }
                }
                AuthState::ChallengeSent(ref nonce) => {
                    match treexml::Element::find_value::<String>(&req, "nonce_hash") {
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
    context: Arc<state::Context<state::ClientState>>,
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
