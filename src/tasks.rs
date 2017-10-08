extern crate std;
extern crate boinc_app_api as api;
extern crate futures;
extern crate futures_cpupool;
extern crate uuid;

use errors;
use util;

use self::futures::*;
use self::future::FutureResult;
use self::futures_cpupool::*;
use self::uuid::*;
use self::std::collections::HashMap;
use self::std::sync::{Arc, Mutex};

use app::*;
use workunit::*;

#[derive(Clone, Copy, Debug)]
pub enum RunStatus {
    Running,
    Stopped,
    Aborted,
    Error,
    Done,
}

struct ProcessData {
    pub pid: i64,
    pub ipcstream: api::IPCStream,
}

enum FullRunStatus {
    Running(ProcessData),
    Stopped,
    Aborted,
    Error,
    Done,
}

impl<'a> From<&'a FullRunStatus> for RunStatus {
    fn from(v: &FullRunStatus) -> Self {
        match *v {
            FullRunStatus::Running(_) => RunStatus::Running,
            FullRunStatus::Stopped => RunStatus::Stopped,
            FullRunStatus::Aborted => RunStatus::Aborted,
            FullRunStatus::Error => RunStatus::Error,
            FullRunStatus::Done => RunStatus::Done,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TaskStatus {
    pub status: RunStatus,
    pub pct_complete: f64,
}

struct Task {
    pub cmdline: String,
    pub status: FullRunStatus,
    pub pct_complete: f64,
}

impl<'a> From<&'a Task> for TaskStatus {
    fn from(v: &Task) -> Self {
        Self {
            status: (&v.status).into(),
            pct_complete: v.pct_complete,
        }
    }
}

/// Managing server that controls all tasks, running or otherwise.
pub trait TaskServer {
    fn tasks(&self) -> errors::FResult<HashMap<Uuid, TaskStatus>>;

    fn create_task(&self, &AppVersion, &Workunit) -> errors::Result<Uuid>;

    fn start_task(&self, &Uuid) -> errors::FResult<()>;
    fn stop_task(&self, &Uuid) -> errors::FResult<()>;
    fn abort_task(&self, &Uuid) -> errors::FResult<()>;
}

pub struct RealTaskServer {
    // Makes sure that the task is alive
    executor: CpuPool,
    monitor_thread: Option<std::thread::JoinHandle<()>>,
    data: Arc<Mutex<HashMap<Uuid, Task>>>,
}

impl TaskServer {
    fn start(&self, id: &Uuid) -> errors::FResult<()> {
        Box::from(future::result(
            Err(errors::ErrorKind::NotImplementedError(()).into()),
        ))
    }
}

impl TaskServer for RealTaskServer {
    fn tasks(&self) -> errors::FResult<HashMap<Uuid, TaskStatus>> {
        Box::from(future::result(Ok(
            self.data
                .lock()
                .unwrap()
                .iter()
                .map(|(id, v)| (id.clone(), v.into()))
                .collect(),
        )))
    }

    fn create_task(&self, app_version: &AppVersion, wu: &Workunit) -> errors::Result<Uuid> {
        let data = self.data.lock().unwrap();

        bail!(errors::ErrorKind::NotImplementedError(()));
    }

    fn start_task(&self, id: &Uuid) -> errors::FResult<()> {
        let data = Arc::clone(&self.data);
        let id = id.clone();

        Box::from(self.executor.spawn(future::lazy(move || {
            let mut d = data.lock().unwrap();
            match d.get_mut(&id) {
                None => Err(errors::ErrorKind::NotImplementedError(()).into()),
                Some(e) => {
                    //e.status = FullRunStatus::Running(ProcessData {});
                    bail!(errors::ErrorKind::NotImplementedError(()));
                }
            }
        })))
    }

    fn stop_task(&self, id: &Uuid) -> errors::FResult<()> {
        Box::from(future::result(
            Err(errors::ErrorKind::NotImplementedError(()).into()),
        ))
    }

    fn abort_task(&self, id: &Uuid) -> errors::FResult<()> {
        Box::from(future::result(
            Err(errors::ErrorKind::NotImplementedError(()).into()),
        ))
    }
}

/// Mock implementation of TaskServer. Useful for testing and development.
#[derive(Debug)]
pub struct MockTaskServer {
    executor: CpuPool,
    data: Mutex<HashMap<Uuid, TaskStatus>>,
}

impl Default for MockTaskServer {
    fn default() -> Self {
        Self {
            executor: CpuPool::new(4),
            data: Default::default(),
        }
    }
}

impl MockTaskServer {
    fn set_status(&self, id: &Uuid, v: RunStatus) -> errors::FResult<()> {
        Box::from(future::result(
            match self.data.lock().unwrap().get_mut(id) {
                None => Err(errors::ErrorKind::NoSuchTaskError(id.clone()).into()),
                Some(info) => {
                    info.status = v;
                    Ok(())
                }
            },
        ))
    }
}

impl TaskServer for MockTaskServer {
    fn tasks(&self) -> errors::FResult<HashMap<Uuid, TaskStatus>> {
        Box::from(future::result(Ok(self.data.lock().unwrap().clone())))
    }

    fn create_task(&self, _: &AppVersion, _: &Workunit) -> errors::Result<Uuid> {
        Ok(
            util::insert_unique(
                &mut (*self.data.lock().unwrap()),
                TaskStatus {
                    status: RunStatus::Stopped,
                    pct_complete: 0.0,
                },
            ).0,
        )
    }

    fn start_task(&self, id: &Uuid) -> errors::FResult<()> {
        self.set_status(id, RunStatus::Running)
    }
    fn stop_task(&self, id: &Uuid) -> errors::FResult<()> {
        self.set_status(id, RunStatus::Stopped)
    }
    fn abort_task(&self, id: &Uuid) -> errors::FResult<()> {
        self.set_status(id, RunStatus::Aborted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
