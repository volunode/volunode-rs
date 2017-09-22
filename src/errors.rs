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
        UserPermissionError(t: String) {
            description("action is not allowed by user"),
            display("action is not allowed by user: {}", &t),
        }
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
        AlreadyAttachedError(t: String) {}
    }
}
