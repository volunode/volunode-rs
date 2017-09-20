extern crate std;

extern crate uuid;

use std::collections::HashMap;

pub fn insert_unique_item<T>(coll: &mut HashMap<uuid::Uuid, T>, v: T) -> uuid::Uuid {
    loop {
        let k = uuid::Uuid::new(uuid::UuidVersion::Random).unwrap();
        if !coll.contains_key(&k) {
            coll.insert(k.clone(), v);
            return k;
        }
    }
}
