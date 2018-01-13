extern crate chrono;
extern crate std;
extern crate treexml;

use std::sync::Arc;

use errors;

pub type Time = chrono::DateTime<chrono::offset::Utc>;
pub type Duration = chrono::Duration;

pub trait ClockSource: Sync + Send {
    fn now(&self) -> Time;
}

#[derive(Copy, Clone, Debug)]
pub struct SystemClockSource;

impl ClockSource for SystemClockSource {
    fn now(&self) -> Time {
        std::time::SystemTime::now().into()
    }
}

pub trait ClockInitializable {
    fn new_with_clock(Arc<ClockSource>) -> Self;
}

#[derive(Clone, Copy, Debug)]
pub enum RunMode {
    Always,
    Auto,
    Never,
    Restore,
}

impl Default for RunMode {
    fn default() -> Self {
        RunMode::Auto
    }
}

impl From<RunMode> for u8 {
    fn from(v: RunMode) -> u8 {
        match v {
            RunMode::Always => 1,
            RunMode::Auto => 2,
            RunMode::Never => 3,
            RunMode::Restore => 4,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NetInfo {
    pub max_rate: f64,
    pub avg_rate: f64,
    pub avg_time: Time,
}

impl NetInfo {
    pub fn update(&self, nbytes: f64, dt: f64) {
        unimplemented!()
    }
}

#[derive(Clone, Debug)]
pub struct NetStats {
    up: NetInfo,
    down: NetInfo,
}

impl NetStats {
    pub fn write(&self) -> treexml::Element {
        unimplemented!()
    }

    pub fn try_from(e: &treexml::Element) -> errors::Result<treexml::Element> {
        unimplemented!()
    }
}

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
    fn master_url(&self) -> String;
    fn project_name(&self) -> Option<String>;

    fn get_project_name(&self) -> String {
        self.project_name().unwrap_or(self.master_url())
    }
}
