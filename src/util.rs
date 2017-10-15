extern crate std;

extern crate uuid;

use std::collections::{HashMap, HashSet};
use self::uuid::Uuid;

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
