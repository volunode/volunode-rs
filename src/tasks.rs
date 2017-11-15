extern crate std;
extern crate boinc_app_api as api;
extern crate futures;
extern crate futures_cpupool;
extern crate futures_spawn;
extern crate uuid;

use errors;
use util;

use self::futures::*;
use self::future::FutureResult;
use self::futures_cpupool::*;
use self::futures_spawn::*;
use self::uuid::*;
use self::std::collections::{HashMap, HashSet};
use self::std::path::PathBuf;
use self::std::sync::{Arc, Mutex, RwLock, atomic};
use self::atomic::{Ordering, AtomicBool};

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
    pub ipcstream: api::IPCStream,
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

    fn create_task(&self, &AppVersion, &Workunit) -> errors::FResult<Uuid>;

    fn start_task(&self, &Uuid) -> errors::FResult<()>;
    fn stop_task(&self, &Uuid) -> errors::FResult<()>;
    fn abort_task(&self, &Uuid) -> errors::FResult<()>;
}

pub struct RealTaskServer {
    // Makes sure that the task is alive
    executor: CpuPool,
    root: std::path::PathBuf,
    worker: Option<std::thread::JoinHandle<()>>,
    data: Arc<Mutex<HashMap<Uuid, Task>>>,
    reserved: Arc<Mutex<HashSet<Uuid>>>,
}

fn task_path(root: &PathBuf, id: &Uuid) -> PathBuf {
    root.join("tasks").join(id.to_string())
}

impl RealTaskServer {
    fn _start_task(&self, id: &Uuid) -> errors::FResult<()> {
        Box::from(future::result(
            Err(errors::ErrorKind::NotImplementedError(()).into()),
        ))
    }

    fn task_path(&self, id: &Uuid) -> PathBuf {
        task_path(&self.root, id)
    }

    fn new(root: std::path::PathBuf) -> Self {
        Self {
            executor: CpuPool::new(4),
            root: root,
            worker: Default::default(),
            data: Default::default(),
            reserved: Default::default(),
        }
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

    fn create_task(&self, app_version: &AppVersion, wu: &Workunit) -> errors::FResult<Uuid> {
        let data = Arc::clone(&self.data);
        let reserved = Arc::clone(&self.reserved);
        let root = self.root.clone();

        Box::from(self.executor.spawn(futures::lazy(move || {
            let id = util::reserve_unique(&data.lock().unwrap(), &mut reserved.lock().unwrap());

            /// Create slot directory
            std::fs::create_dir_all(task_path(&root, &id));

            /// Copy task files: appversion, wu, etc

            Err(errors::ErrorKind::NotImplementedError(()).into())
        })))
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
                    let mut d: std::sync::MutexGuard<HashMap<Uuid, TaskStatus>> = data.lock()
                        .unwrap();

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

    fn create_task(&self, _: &AppVersion, _: &Workunit) -> errors::FResult<Uuid> {
        Box::from(future::result(Ok(
            util::insert_unique(
                &mut (*self.data.lock().unwrap()),
                TaskStatus {
                    status: RunStatus::Stopped,
                    pct_complete: 0.0,
                },
            ).0,
        )))
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
