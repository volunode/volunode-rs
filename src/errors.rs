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
    }
    errors {
        ConnectError(t: String) {
            description(""),
            display("{}", t),
        }
        //DataParseError(_: String) {}
        //InvalidPasswordError(_: String) {}
        //DaemonError(_: String) {}
        //NullError(_: String) {}
        //NetworkError(_: String) {}
        //StatusError(_: i32) {}
        //AuthError(_: String) {}
        //InvalidURLError(_: String) {}
        //AlreadyAttachedError(_: String) {}
    }
}
