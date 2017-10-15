extern crate std;
extern crate chan;
extern crate uuid;

use errors;

use self::std::io::prelude::*;
use self::std::process::{Command, Stdio};
use self::std::sync::{Arc, RwLock, Mutex, mpsc};
use self::std::sync::atomic::{AtomicBool, Ordering};
use self::std::io::BufReader;
use self::uuid::Uuid;

use app::*;
use errors::*;
use workunit::*;

pub type ProcessOutputCB = Arc<Fn(String) + Send + Sync + 'static>;

pub trait Process {
    fn push(&self, String);
    fn set_output_cb(&self, Option<ProcessOutputCB>);
    fn get_output_cb(&self) -> Option<ProcessOutputCB>;
}

pub struct SystemProcess {
    dropping: Arc<AtomicBool>,

    process_manager: Option<std::thread::JoinHandle<()>>,
    output_cb: Arc<Mutex<Option<ProcessOutputCB>>>,
    input: Arc<RwLock<Option<std::process::ChildStdin>>>,
}

impl Drop for SystemProcess {
    fn drop(&mut self) {
        self.dropping.store(true, Ordering::Relaxed);
        self.process_manager.take().unwrap().join();
    }
}

impl Process for SystemProcess {
    fn push(&self, buf: String) {
        let input = Arc::clone(&self.input);
        let dropping = Arc::clone(&self.dropping);
        std::thread::spawn(move || loop {
            match input.write().unwrap().as_mut() {
                Some(s) => {
                    s.write_all(buf.as_bytes());
                }
                None => {
                    if dropping.load(Ordering::Relaxed) {
                        return;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(300));

                }
            }
        });
    }

    fn set_output_cb(&self, mut output_cb: Option<ProcessOutputCB>) {
        let mut obj = self.output_cb.lock().unwrap();
        std::mem::swap(&mut *obj, &mut output_cb);
    }

    fn get_output_cb(&self) -> Option<ProcessOutputCB> {
        self.output_cb.lock().unwrap().clone()
    }
}

impl SystemProcess {
    pub fn new<S1, S2>(procname: S1, args: S2) -> Self
    where
        S1: ToString,
        S2: ToString,
    {
        let dropping = Arc::new(AtomicBool::new(false));
        let output_cb = Arc::new(Mutex::new(None));
        let input = Arc::new(RwLock::new(None));
        let process_manager = std::thread::spawn({
            let procname = procname.to_string();
            let args = args.to_string();
            let dropping = Arc::clone(&dropping);
            let input = Arc::clone(&input);
            let output_cb = Arc::clone(&output_cb);
            move || {
                let mut process = None;
                let mut it: Option<std::io::Lines<BufReader<std::process::ChildStdout>>> = None;

                loop {
                    if dropping.load(Ordering::Relaxed) {
                        break;
                    }
                    match process {
                        None => {
                            let mut p = Command::new(&procname)
                                .arg(&args)
                                .stdin(Stdio::piped())
                                .stdout(Stdio::inherit())
                                .spawn()
                                .unwrap();
                            let in_s = p.stdin.take();
                            it = Some(BufReader::new(p.stdout.take().unwrap()).lines());
                            process = Some(p);
                            *input.write().unwrap() = in_s;
                        }
                        Some(_) => {
                            for line in it.take().unwrap() {
                                match line {
                                    Ok(v) => {
                                        {
                                            let o = output_cb.lock().unwrap();
                                            let fopt: Option<&ProcessOutputCB> = o.as_ref();
                                            if let Some(f) = fopt {
                                                f(v);
                                            }
                                        }
                                        std::thread::sleep(std::time::Duration::from_millis(300));
                                    }
                                    Err(err) => {
                                        if let std::io::ErrorKind::BrokenPipe = err.kind() {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    *input.write().unwrap() = None;
                    process = None;
                }

                process.take().unwrap().kill();
            }
        });

        Self {
            dropping: dropping,
            process_manager: Some(process_manager),
            input: input,
            output_cb: output_cb,
        }
    }
}
