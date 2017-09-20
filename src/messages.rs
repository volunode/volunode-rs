extern crate std;
extern crate treexml;
extern crate treexml_util;

use std::sync::{Arc, RwLock};
use std::fmt::Display;

use common;

#[derive(Clone, Debug)]
pub struct Message {
    pub project_name: Option<String>,
    pub priority: common::MessagePriority,
    pub body: String,
    pub timestamp: common::Time,
}

impl<'a> From<&'a Message> for treexml::Element {
    fn from(v: &Message) -> treexml::Element {
        treexml::Element {
            name: "msg".into(),
            children: vec![
                treexml_util::serialize_node(
                    "project",
                    v.project_name.as_ref().or(Some(&"".into())).unwrap()
                ),
                treexml_util::serialize_node("pri", &u8::from(v.priority.clone())),
                treexml_util::serialize_cdata("body", &v.body),
                treexml_util::serialize_node("time", &v.timestamp.timestamp()),
            ],
            ..Default::default()
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{} [{}] {}",
            self.timestamp,
            match self.project_name.as_ref() {
                Some(s) => s.as_str(),
                None => "---",
            },
            self.body
        );
        Ok(())
    }
}

pub trait Logger {
    fn insert(&self, Option<&common::ProjAm>, common::MessagePriority, common::Time, &str);
    fn cleanup(&self);
    fn len(&self) -> usize;
    fn get(&self, start: usize) -> Vec<Message>;

    fn to_xml(&self, seqno: Option<usize>) -> treexml::ElementBuilder {
        let mut out = treexml::Element::new("msgs");
        out.children = self.get(seqno.or(Some(1)).unwrap())
            .into_iter()
            .enumerate()
            .map(|(i, msg)| {
                let mut e = treexml::Element::from(&msg);
                e.children.push(
                    treexml_util::serialize_node("seqno", &(i + 1)),
                );
                e
            })
            .collect();
        out.into()
    }
}

pub type SafeLogger = Arc<Logger + Send + Sync>;

#[derive(Debug, Default)]
pub struct StandardLogger {
    msgs: RwLock<Vec<Message>>,
}

impl Logger for StandardLogger {
    fn insert(
        &self,
        project: Option<&common::ProjAm>,
        priority: common::MessagePriority,
        now: common::Time,
        msg: &str,
    ) {
        let m = Message {
            project_name: project.map(|p| p.get_project_name().into()),
            priority: priority,
            body: msg.into(),
            timestamp: now,
        };
        let msgs = &mut *self.msgs.write().unwrap();
        let s = format!("{}", &m);

        msgs.push(m);
        println!("{}", s);
    }

    fn cleanup(&self) {
        self.msgs.write().unwrap().clear();
    }

    fn len(&self) -> usize {
        self.msgs.read().unwrap().len()
    }

    fn get(&self, seqno: usize) -> Vec<Message> {
        let data = self.msgs.read().unwrap();
        if seqno >= data.len() {
            vec![]
        } else {
            match data.get(if seqno < 1 { 1 } else { seqno } - 1..data.len()) {
                Some(out) => out.into(),
                None => vec![],
            }
        }
    }
}

impl Clone for StandardLogger {
    fn clone(&self) -> StandardLogger {
        StandardLogger { msgs: RwLock::new(self.msgs.read().unwrap().clone()) }
    }
}
