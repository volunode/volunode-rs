extern crate std;

use std::io::prelude::*;
use std::process::{Command, Stdio};
use std::sync::{Arc, RwLock, atomic, mpsc};
use std::thread;
use std::io::BufReader;

#[derive(Debug)]
pub struct Process {
    dropping: Arc<std::sync::atomic::AtomicBool>,
    process_manager: Option<std::thread::JoinHandle<()>>,
    input: Arc<RwLock<Option<std::process::ChildStdin>>>,
}

impl Drop for Process {
    fn drop(&mut self) {
        self.dropping.store(true, atomic::Ordering::Relaxed);
        self.process_manager.take().unwrap().join();
    }
}

impl Process {
    pub fn new<S1, S2, F>(procname: S1, args: S2, out_cb: F) -> Process
    where
        S1: ToString,
        S2: ToString,
        F: Fn(String) + Send + 'static,
    {
        let dropping = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let input = Arc::new(RwLock::new(None));
        let process_manager = std::thread::spawn({
            let procname = procname.to_string();
            let args = args.to_string();
            let dropping = dropping.clone();
            let input = input.clone();
            move || {
                let mut process = None;
                let mut it: Option<std::io::Lines<BufReader<std::process::ChildStdout>>> = None;

                loop {
                    if dropping.load(atomic::Ordering::Relaxed) {
                        break;
                    }
                    match &mut process {
                        &mut None => {
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
                        &mut Some(_) => {
                            for line in it.take().unwrap() {
                                match line {
                                    Ok(v) => {
                                        out_cb(v);
                                        std::thread::sleep(std::time::Duration::from_millis(300));
                                    }
                                    Err(err) => {
                                        match err.kind() {
                                            std::io::ErrorKind::BrokenPipe => {
                                                break;
                                            }
                                            _ => {}
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

        Process {
            dropping: dropping,
            process_manager: Some(process_manager),
            input: input,
        }
    }

    pub fn push(&mut self, buf: Vec<u8>) {
        let input = self.input.clone();
        let dropping = self.dropping.clone();
        std::thread::spawn(move || loop {
            match input.write().unwrap().as_mut() {
                Some(s) => {
                    s.write_all(&buf);
                    return;
                }
                None => {
                    if dropping.load(std::sync::atomic::Ordering::Relaxed) {
                        return;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(300));

                }
            }
        });
    }
}
