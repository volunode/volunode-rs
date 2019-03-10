extern crate std;

extern crate boinc_app_api as api;
extern crate futures;
extern crate treexml;
extern crate uuid;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "requested operation is stubbed: {}", name)]
    NotImplementedError { name: String },
    #[fail(display = "{}", t)]
    ConnectError { t: String },
    #[fail(display = "task {} does not exist", id)]
    NoSuchTaskError { id: uuid::Uuid },
    #[fail(display = "already attached")]
    AlreadyAttachedError,
    //DataParseError(_: String) {}
    //InvalidPasswordError(_: String) {}
    //DaemonError(_: String) {}
    //NullError(_: String) {}
    //NetworkError(_: String) {}
    //StatusError(_: i32) {}
    #[fail(display = "authentication error: {}", what)]
    AuthError { what: String },
    #[fail(display = "invalid URL: {}", url)]
    InvalidURLError { url: String },
    #[fail(display = "action is not allowed by user: {}", what)]
    UserPermissionError { what: String },
    #[fail(display = "internal error has occurred: {}", what)]
    InternalError { what: String },
    #[fail(display = "data parsing failed: {}", what)]
    DataParseError { what: String },
    #[fail(display = "API error: {}", inner)]
    APIError { inner: api::Error },
}

impl Error {
    pub fn rpc_id(&self) -> i64 {
        match *self {
            Error::AlreadyAttachedError => -130,
            Error::AuthError {} => -155,
            Error::InvalidURLError {} => -189,
            Error::UserPermissionError {} => -201,
            _ => -1,
        }
    }
}

pub type R<T> = Result<T, Error>;
pub type FResult<T> = Box<futures::Future<Item = T, Error = Error>>;
