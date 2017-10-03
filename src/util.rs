extern crate std;

extern crate uuid;

use std::collections::HashMap;

pub fn insert_unique<T>(coll: &mut HashMap<uuid::Uuid, T>, v: T) -> (uuid::Uuid, &mut T) {
    loop {
        let k = uuid::Uuid::new(uuid::UuidVersion::Random).unwrap();
        if !coll.contains_key(&k) {
            coll.insert(k, v);
            return (k, coll.get_mut(&k).unwrap());
        }
    }
}

pub fn canonicalize_url(s: &str) -> String {
    String::from(s).replace("/", "_")
}
