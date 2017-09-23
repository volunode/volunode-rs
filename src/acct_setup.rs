extern crate std;

use errors;

#[derive(Debug, Default)]
pub struct ProjectAttach {
    pub error: std::sync::Mutex<Option<errors::Error>>,
    pub messages: Vec<String>,
}
