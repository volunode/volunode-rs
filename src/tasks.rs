extern crate std;
extern crate uuid;

use errors;
use util;

use self::uuid::*;
use self::std::collections::HashMap;
use self::std::sync::Mutex;

use app::*;
use workunit::*;

#[derive(Clone, Copy, Debug)]
pub enum RunStatus {
    Running,
    Stopped,
    Aborted,
    Done,
}

#[derive(Clone, Debug)]
pub struct TaskStatus {
    status: RunStatus,
    pct_complete: f64,
}

pub trait TaskServer {
    fn tasks(&self) -> HashMap<Uuid, TaskStatus>;

    fn create_task(&self, &AppVersion, &Workunit) -> errors::Result<Uuid>;

    fn start_task(&self, &Uuid) -> bool;
    fn stop_task(&self, &Uuid) -> bool;
    fn abort_task(&self, &Uuid) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    pub struct MockTaskServer {
        data: Mutex<HashMap<Uuid, TaskStatus>>,
    }


    impl MockTaskServer {
        fn set_status(&self, id: &Uuid, v: RunStatus) -> bool {
            match self.data.lock().unwrap().get_mut(id) {
                None => false,
                Some(info) => {
                    info.status = v;
                    true
                }
            }
        }
    }

    impl TaskServer for MockTaskServer {
        fn tasks(&self) -> HashMap<Uuid, TaskStatus> {
            self.data.lock().unwrap().clone()
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

        fn start_task(&self, id: &Uuid) -> bool {
            self.set_status(id, RunStatus::Running)
        }
        fn stop_task(&self, id: &Uuid) -> bool {
            self.set_status(id, RunStatus::Stopped)
        }
        fn abort_task(&self, id: &Uuid) -> bool {
            self.set_status(id, RunStatus::Aborted)
        }
    }
}
