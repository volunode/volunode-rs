extern crate chrono;
extern crate std;

use std::sync::{Arc, Mutex};

pub type Time = chrono::DateTime<chrono::offset::Utc>;
pub type ClockSource = Arc<Fn() -> Time + Sync + Send>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RpcReason {
    UserRequest,
    ResultsDue,
    NeedWork,
    TrickleUp,
    AccountManagerRequest,
    Init,
    ProjectRequest,
}

impl From<RpcReason> for u8 {
    fn from(v: RpcReason) -> u8 {
        match v {
            RpcReason::UserRequest => 1,
            RpcReason::ResultsDue => 2,
            RpcReason::NeedWork => 3,
            RpcReason::TrickleUp => 4,
            RpcReason::AccountManagerRequest => 5,
            RpcReason::Init => 6,
            RpcReason::ProjectRequest => 7,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    fn master_url(&self) -> &str;
    fn project_name(&self) -> Option<&str>;

    fn get_project_name(&self) -> &str {
        self.project_name().or(Some(self.master_url())).unwrap()
    }
}
