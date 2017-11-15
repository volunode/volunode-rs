extern crate std;
extern crate futures;
extern crate futures_cpupool;
extern crate futures_spawn;
extern crate hyper;
extern crate reqwest;
extern crate tokio_core;
extern crate uuid;

use errors;
use util;

use self::futures::*;
use self::future::FutureResult;
use self::futures_cpupool::*;
use self::futures_spawn::*;
use self::uuid::*;
use self::std::collections::{HashMap, HashSet};
use self::std::path::PathBuf;
use self::std::sync::{Arc, Mutex, RwLock, atomic};
use self::atomic::{Ordering, AtomicBool};
use self::reqwest::unstable::async::Client as AsyncClient;
use self::reqwest::unstable::async::Request as AsyncRequest;
use self::tokio_core::reactor::Core as IOCore;
use self::std::io::Write;

use app::*;
use common::*;
use workunit::*;

pub enum XferDirection {
    Up,
    Down,
}

pub struct Xfer {
    worker: Option<errors::FResult<()>>,
}

pub struct XferStatus {}

#[derive(Default)]
pub struct XferServerStatus {
    pub up: u64,
    pub down: u64,
    pub current_xfers: HashMap<Uuid, XferStatus>,
}

pub trait XferServer {
    fn status(&self) -> XferServerStatus;

    fn add(&self, req: AsyncRequest, out: Box<Write>) -> errors::FResult<Uuid>;
    fn remove(&self, id: &Uuid) -> errors::FResult<()>;
    fn start(&self, id: &Uuid) -> errors::FResult<()>;
    fn stop(&self, id: &Uuid) -> errors::FResult<()>;
}

pub struct RealXferServer {
    iocore: Arc<IOCore>,
    client: Arc<AsyncClient>,
    xfers: Arc<Mutex<HashMap<Uuid, Xfer>>>,
}

impl Default for RealXferServer {
    fn default() -> Self {
        let iocore = Arc::new(IOCore::new().unwrap());
        Self {
            iocore: Arc::clone(&iocore),
            client: Arc::new(AsyncClient::new(&iocore.handle())),
            xfers: Default::default(),
        }
    }
}

impl XferServer for RealXferServer {
    fn status(&self) -> XferServerStatus {
        Default::default()
    }

    fn add(&self, req: AsyncRequest, out: Box<Write>) -> errors::FResult<Uuid> {
        let client = Arc::clone(&self.client);
        let xfers = Arc::clone(&self.xfers);
        errors::fspawn(move || {
            let xfer_data = xfers.lock().unwrap();
            let id = util::insert_unique(
                &mut *xfer_data,
                Xfer {
                    worker: Some(errors::fspawn(move || {
                        client.execute(req);

                        loop {}
                    })),
                },
            );

            Ok(id.0)
        })
    }

    fn remove(&self, id: &Uuid) -> errors::FResult<()> {
        unimplemented!()
    }

    fn start(&self, id: &Uuid) -> errors::FResult<()> {
        unimplemented!()
    }

    fn stop(&self, id: &Uuid) -> errors::FResult<()> {
        unimplemented!()
    }
}
