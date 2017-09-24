extern crate std;

extern crate error_chain;
extern crate treexml;

extern crate boinc_app_api;

error_chain! {
    links {
        APIError(boinc_app_api::Error, boinc_app_api::ErrorKind);
        XMLError(treexml::Error, treexml::ErrorKind);
    }
    foreign_links {
        StringConversionError(std::string::FromUtf8Error);
        IOError(std::io::Error);
    }
    errors {
        ConnectError(t: String) {
            description(""),
            display("{}", t),
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
