extern crate std;

extern crate futures;
extern crate uuid;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, TryLockError};
use self::futures::prelude::*;
use self::futures::future::poll_fn;
use self::uuid::Uuid;

use errors;

pub fn insert_unique<T>(coll: &mut HashMap<uuid::Uuid, T>, v: T) -> (uuid::Uuid, &mut T) {
    loop {
        let k = uuid::Uuid::new(uuid::UuidVersion::Random).unwrap();
        if !coll.contains_key(&k) {
            coll.insert(k, v);
            return (k, coll.get_mut(&k).unwrap());
        }
    }
}

pub fn reserve_unique<T>(coll: &HashMap<Uuid, T>, reserve: &mut HashSet<Uuid>) -> uuid::Uuid {
    loop {
        let k = uuid::Uuid::new(uuid::UuidVersion::Random).unwrap();
        if !coll.contains_key(&k) && !reserve.contains(&k) {
            reserve.insert(k);
            return k;
        }
    }
}

pub fn canonicalize_url(s: &str) -> String {
    String::from(s).replace("/", "_")
}

pub fn task_path(root: &PathBuf, id: &Uuid) -> PathBuf {
    root.join("tasks").join(id.to_string())
}

pub fn mutex_critical<T, U, F>(
    data: Arc<Mutex<T>>,
    f: F,
) -> impl Future<Item = U, Error = errors::Error>
where
    F: Fn(&mut T) -> errors::Result<U>,
{
    poll_fn(move || match data.try_lock() {
        Err(e) => match e {
            TryLockError::WouldBlock => Ok(Async::NotReady),
            TryLockError::Poisoned(m) => {
                Err(errors::ErrorKind::InternalError("Poisoned mutex".into()).into())
            }
        },
        Ok(mut g) => f(&mut *g).map(|v| Async::Ready(v)),
    })
}
