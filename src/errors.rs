extern crate std;

extern crate error_chain;
extern crate futures;
extern crate futures_spawn;
extern crate treexml;
extern crate uuid;

extern crate boinc_app_api;

use self::futures_spawn::SpawnHelper;

error_chain! {
    links {
        APIError(boinc_app_api::errors::Error, boinc_app_api::errors::ErrorKind);
    }
    foreign_links {
        StringConversionError(std::string::FromUtf8Error);
        IOError(std::io::Error);
        XMLError(treexml::Error);
    }
    errors {
        NotImplementedError(t: ()) {
            description("not implemented"),
            display("requested operation is stubbed but will be implemented in the future"),
        }
        ConnectError(t: String) {
            description(""),
            display("{}", t),
        }
        NoSuchTaskError(id: uuid::Uuid) {
            description("no such task"),
            display("Task {} does not exist", id),
        }
        AlreadyAttachedError(t: String) {}
        //DataParseError(_: String) {}
        //InvalidPasswordError(_: String) {}
        //DaemonError(_: String) {}
        //NullError(_: String) {}
        //NetworkError(_: String) {}
        //StatusError(_: i32) {}
        AuthError(t: String) {
            description("authentication error"),
            display("authentication error"),
        }
        InvalidURLError(t: String) {
            description("invalid URL"),
            display("invalid URL: {}", &t),
        }
        UserPermissionError(t: String) {
            description("action is not allowed by user"),
            display("action is not allowed by user: {}", &t),
        }
    }
}

impl<'a> From<&'a Error> for i64 {
    fn from(v: &Error) -> i64 {
        match v.kind() {
            &ErrorKind::AlreadyAttachedError(_) => -130,
            &ErrorKind::AuthError(_) => -155,
            &ErrorKind::InvalidURLError(_) => -189,
            &ErrorKind::UserPermissionError(_) => -201,
            _ => -1,
        }
    }
}

pub type FResultUnboxed<T> = futures::future::Future<Item = T, Error = Error>;
pub type FResult<T> = Box<futures::future::Future<Item = T, Error = Error>>;
pub type TResult<T> = std::thread::JoinHandle<self::Result<T>>;

/// Spawns a new thread and returns a future.
pub fn fspawn<T, F>(f: F) -> FResult<T>
where
    T: Send + 'static,
    F: Fn() -> Result<T> + Send + 'static,
{
    Box::from(futures_spawn::NewThread.spawn(futures::lazy(f)))
}
