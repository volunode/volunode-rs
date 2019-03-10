#![feature(proc_macro, conservative_impl_trait, generators)]

extern crate boinc_app_api as api;
extern crate futures_await as futures;
extern crate futures_cpupool;
extern crate futures_spawn;
extern crate std;
extern crate uuid;

use errors;
use util;

use self::futures::future::{ok, FutureResult, PollFn};
use self::futures::prelude::*;
use self::futures::*;
use self::futures_cpupool::*;
use self::futures_spawn::*;
use self::uuid::*;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock, TryLockError};

use app::*;
use workunit::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunStatus {
    Running,
    StopRequested,
    Stopped,
    Aborted,
    Error,
    Done,
}

struct ProcessData {
    pub pid: i64,
    pub conn: api::app_connection::AppConnection,
}

enum FullRunStatus {
    Running(ProcessData),
    StopRequested(ProcessData),
    Stopped,
    Aborted,
    Error,
    Done,
}

impl<'a> From<&'a FullRunStatus> for RunStatus {
    fn from(v: &FullRunStatus) -> Self {
        match *v {
            FullRunStatus::Running(_) => RunStatus::Running,
            FullRunStatus::StopRequested(_) => RunStatus::StopRequested,
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

impl Task {
    pub fn get_status(&self) -> TaskStatus {
        TaskStatus {
            status: (&self.status).into(),
            pct_complete: self.pct_complete,
        }
    }
}

/*
impl Future for Task {
    type Item = ();
    type Error = errors::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.status {
            Done => Async::Ready(()),
        }
    }
}
*/

/// Managing server that controls all tasks, running or otherwise.
pub trait TaskServer {
    fn tasks(&self) -> errors::FResult<HashMap<Uuid, TaskStatus>>;

    fn create_task(&self, &AppVersion, &Workunit) -> errors::FResult<Uuid>;

    fn start_task(&self, &Uuid) -> errors::FResult<()>;
    fn stop_task(&self, &Uuid) -> errors::FResult<()>;
    fn abort_task(&self, &Uuid) -> errors::FResult<()>;
}

pub struct RealTaskServer {
    // Makes sure that the task is alive
    root: std::path::PathBuf,
    worker: Option<std::thread::JoinHandle<()>>,
    data: Arc<Mutex<HashMap<Uuid, Task>>>,
    reserved: Arc<Mutex<HashSet<Uuid>>>,
}

impl RealTaskServer {
    fn task_path(&self, id: &Uuid) -> PathBuf {
        util::task_path(&self.root, id)
    }

    fn new(root: std::path::PathBuf) -> Self {
        Self {
            root: root,
            worker: Default::default(),
            data: Default::default(),
            reserved: Default::default(),
        }
    }

    #[async(boxed)]
    fn _create_task(
        data: Arc<Mutex<HashMap<Uuid, Task>>>,
        reserved: Arc<Mutex<HashSet<Uuid>>>,
        root: std::path::PathBuf,
        app_version: &AppVersion,
        wu: &Workunit,
    ) -> errors::R<Uuid> {
        let id = await!(util::mutex_critical(data, move |data| Ok(
            util::reserve_unique(data, &mut reserved.lock().unwrap())
        )))?;

        // Create slot directory
        std::fs::create_dir_all(util::task_path(&root, &id));

        // Copy task files: appversion, wu, etc
        Err(errors::Error::NotImplementedError {
            name: "create_task".into(),
        })
    }
}

impl TaskServer for RealTaskServer {
    fn tasks(&self) -> errors::FResult<HashMap<Uuid, TaskStatus>> {
        Box::new(util::mutex_critical(Arc::clone(&self.data), |data| {
            Ok(data
                .iter()
                .map(|(id, v)| (id.clone(), v.get_status()))
                .collect())
        }))
    }

    fn create_task(&self, app_version: &AppVersion, wu: &Workunit) -> errors::FResult<Uuid> {
        Self::_create_task(
            self.data.clone(),
            self.reserved.clone(),
            self.root.clone(),
            app_version,
            wu,
        )
    }

    fn start_task(&self, id: &Uuid) -> errors::FResult<()> {
        let id = id.clone();
        Box::new(util::mutex_critical(
            Arc::clone(&self.data),
            move |d| match d.get_mut(&id) {
                None => Err(errors::Error::NoSuchTaskError { id: id }),
                Some(e) => {
                    //e.status = FullRunStatus::Running(ProcessData {});
                    Err(errors::Error::NotImplementedError {
                        name: "starting a task".into(),
                    })
                }
            },
        ))
    }

    fn stop_task(&self, id: &Uuid) -> errors::FResult<()> {
        Box::new(async_block! {
            Err(errors::Error::NotImplementedError { name: "stopping a task".into() })
        })
    }

    fn abort_task(&self, id: &Uuid) -> errors::FResult<()> {
        Box::new(async_block! {
            Err(errors::Error::NotImplementedError { name: "aborting a task".into() })
        })
    }
}

struct MockTask {
    pub status: Arc<RwLock<TaskStatus>>,
    close_chan: Option<std::sync::mpsc::Sender<()>>,
    refresher_thread: Arc<Option<std::thread::JoinHandle<()>>>,
}

/// Mock implementation of TaskServer. Useful for testing and development.
pub struct MockTaskServer {
    executor: CpuPool,
    close_flag: Arc<AtomicBool>,
    data: Arc<Mutex<HashMap<Uuid, TaskStatus>>>,
    worker: Option<std::thread::JoinHandle<()>>,
}

fn progress_mock_tasks(data: &mut HashMap<Uuid, TaskStatus>, pace: f64) {
    for (_, v) in data.iter_mut() {
        if v.status == RunStatus::Running {
            v.pct_complete += pace;

            if v.pct_complete >= 1.0 {
                v.pct_complete = 1.0;
                v.status = RunStatus::Done;
            }
        }
    }
}

impl Default for MockTaskServer {
    fn default() -> Self {
        let data = Arc::new(Mutex::new(HashMap::default()));
        let executor = CpuPool::new(4);

        let close_flag = Arc::new(AtomicBool::default());

        let worker = std::thread::spawn({
            let data = Arc::clone(&data);
            let close_flag = Arc::clone(&close_flag);

            move || loop {
                if close_flag.load(Ordering::Relaxed) == true {
                    return;
                }
                {
                    let mut d: std::sync::MutexGuard<HashMap<Uuid, TaskStatus>> =
                        data.lock().unwrap();

                    progress_mock_tasks(&mut *d, 0.04);
                }
                std::thread::sleep(std::time::Duration::from_secs(3));
            }
        });

        Self {
            executor: executor,
            data: data,
            close_flag: close_flag,
            worker: Some(worker),
        }
    }
}

impl Drop for MockTaskServer {
    fn drop(&mut self) {
        self.close_flag.store(true, Ordering::Relaxed);
        self.worker.take().unwrap().join().unwrap();
    }
}

impl MockTaskServer {
    fn set_status(&self, id: &Uuid, v: RunStatus) -> errors::FResult<()> {
        let id = id.clone();
        Box::new(util::mutex_critical(
            Arc::clone(&self.data),
            move |data| match data.get_mut(&id) {
                None => Err(errors::Error::NoSuchTaskError { id }),
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
        Box::new(util::mutex_critical(Arc::clone(&self.data), |data| {
            Ok(data.clone())
        }))
    }

    fn create_task(&self, _: &AppVersion, _: &Workunit) -> errors::FResult<Uuid> {
        Box::new(util::mutex_critical(Arc::clone(&self.data), |data| {
            Ok(util::insert_unique(
                data,
                TaskStatus {
                    status: RunStatus::Stopped,
                    pct_complete: 0.0,
                },
            )
            .0)
        }))
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
