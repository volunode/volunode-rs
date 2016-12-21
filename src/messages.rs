extern crate std;
extern crate treexml;

use std::sync::{Arc, RwLock};

use common;

#[derive(Clone, Debug)]
pub struct Message {
    pub project_name: Option<String>,
    pub priority: common::MessagePriority,
    pub body: String,
    pub timestamp: common::Time,
}

pub trait Logger : Sync {
    fn insert(&self, Option<&common::ProjAm>, common::MessagePriority, common::Time, &str);
    fn cleanup(&self);
    fn len(&self) -> usize;
    fn get(&self, start: usize) -> Vec<Message>;
}

pub type SafeLogger = Arc<Logger + Send + Sync>;

#[derive(Debug, Default)]
pub struct MessageDescs {
    msgs: RwLock<Vec<Message>>,
}

unsafe impl Sync for MessageDescs {}

impl Logger for MessageDescs {
    fn insert(
        &self,
        project: Option<&common::ProjAm>,
        priority: common::MessagePriority,
        now: common::Time,
        msg: &str,
    ) {
        self.msgs.write().unwrap().push(Message {
            project_name: project.map(|p| p.get_project_name().into()),
            priority: priority,
            body: msg.into(),
            timestamp: now,
        })
    }

    fn cleanup(&self) {
        self.msgs.write().unwrap().clear();
    }

    fn len(&self) -> usize {
        self.msgs.read().unwrap().len()
    }

    fn get(&self, start: usize) -> Vec<Message> {
        let data = self.msgs.read().unwrap();
        match data.get(start..data.len()-1) {
            Some(out) => {
                out.into()
            }
            None => vec![]
        }
    }
}

impl Clone for MessageDescs {
    fn clone(&self) -> MessageDescs {
        MessageDescs {
            msgs: RwLock::new(self.msgs.read().unwrap().clone())
        }
    }
}
