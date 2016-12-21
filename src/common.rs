extern crate chrono;
extern crate std;

use std::sync::Arc;

pub type Time = chrono::DateTime<chrono::offset::Utc>;
pub type ClockSource = Arc<Fn() -> Time + Sync + Send>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Debug,
    Info,
    UserAlert,
    InternalError,
    SchedulerAlert,
}

impl From<MessagePriority> for u8 {
    fn from(v: MessagePriority) -> u8 {
        match v {
            MessagePriority::Debug => 0,
            MessagePriority::Info => 1,
            MessagePriority::UserAlert => 2,
            MessagePriority::InternalError => 3,
            MessagePriority::SchedulerAlert => 4,
        }
    }
}


impl MessagePriority {
    fn from_num(v: u8) -> Option<MessagePriority> {
        if false {
            None
        } else if v == 0 {
            Some(MessagePriority::Debug)
        } else if v == 1 {
            Some(MessagePriority::Info)
        } else if v == 2 {
            Some(MessagePriority::UserAlert)
        } else if v == 3 {
            Some(MessagePriority::InternalError)
        } else if v == 4 {
            Some(MessagePriority::SchedulerAlert)
        } else {
            None
        }
    }
}

pub trait ProjAm {
    fn master_url(&self) -> String;
    fn project_name(&self) -> Option<String>;

    fn get_project_name(&self) -> String {
        self.project_name().or(Some(self.master_url())).unwrap()
    }
}
