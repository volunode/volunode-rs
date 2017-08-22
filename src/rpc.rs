extern crate bytes;
extern crate crypto;
extern crate futures;
extern crate futures_cpupool;
extern crate std;
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

use std::io;
use std::sync::{Arc, RwLock};

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
struct TcpService {
    cpu_pool: futures_cpupool::CpuPool,

    state: Arc<RwLock<state::ClientState>>,
    conn_status: Arc<RwLock<AuthState>>,

    rpcpass: Option<String>,
}

impl TcpService {
    fn new(
        state: Arc<RwLock<state::ClientState>>,
        cpu_pool: futures_cpupool::CpuPool,
        pass: Option<String>,
    ) -> TcpService {
        TcpService {
            cpu_pool: cpu_pool,
            state: state,
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

    fn process_request(&self, v: &treexml::Element) -> treexml::Element {
        match &*v.name {
            "get_message_count" => {
                treexml::ElementBuilder::new("seqno")
                    .text(format!("{}", self.state.read().unwrap().messages.len()))
                    .element()
            }
            "get_messages" => {
                println!("received get_messages");
                let seqno = v.find_value("seqno").unwrap_or(None);
                self.state.read().unwrap().messages.to_xml(seqno).element()
            }
            "get_state" => (&*self.state.read().unwrap()).into(),
            _ => v.clone(),
        }
    }
}

impl Service for TcpService {
    type Request = treexml::Element;
    type Response = treexml::Element;

    type Error = io::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn call(&self, full_request: Self::Request) -> Self::Future {
        let rsp_ok = |mut v: treexml::ElementBuilder| {
            future::ok(
                treexml::ElementBuilder::new("boinc_gui_rpc_reply")
                    .children(vec![&mut v])
                    .element(),
            ).boxed()
        };
        let rsp_err = |v| future::err(v).boxed();

        let unauthorize = |s: &mut AuthState| {
            *s = AuthState::Unauthorized;
            treexml::ElementBuilder::new("unauthorized")
        };
        let authorize = |s: &mut AuthState| {
            *s = AuthState::Ready;
            treexml::ElementBuilder::new("authorized")
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

                                rsp_ok(v)
                            }
                            &None => rsp_ok(authorize(&mut *s)),
                        }
                    } else {
                        rsp_ok(unauthorize(&mut *s))
                    }
                }
                AuthState::ChallengeSent(ref nonce) => {
                    match treexml::Element::find_value::<String>(&req, "nonce_hash") {
                        Ok(opt_response) => {
                            match opt_response {
                                Some(response) => {
                                    if self.salted_hash(nonce).unwrap() == response {
                                        rsp_ok(authorize(&mut *s))
                                    } else {
                                        rsp_ok(unauthorize(&mut *s))
                                    }
                                }
                                None => rsp_ok(unauthorize(&mut *s)),
                            }
                        }
                        Err(_) => rsp_ok(unauthorize(&mut *s)),
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

pub fn StartRpcServer(
    client_state: Arc<RwLock<state::ClientState>>,
    addr: std::net::SocketAddr,
    password: Option<String>,
) -> () {
    let server = tokio_proto::TcpServer::new(RPCProto, addr);
    let thread_pool = futures_cpupool::CpuPool::new(10);
    let p = password.clone();
    let cb = move || {
        Ok(TcpService::new(
            client_state.clone(),
            thread_pool.clone(),
            p.clone(),
        ))
    };
    server.serve(cb);
}
