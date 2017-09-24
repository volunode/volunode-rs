extern crate std;

extern crate uuid;

use cert_sig;
use common;

#[derive(Debug)]
pub struct FileInfo {
    pub name: String,
    pub md5_cksum: String,
    pub max_nbytes: f64,
    pub nbytes: f64,
    pub gzipped_nbytes: f64,
    pub upload_offset: f64,
    pub status: i64,
    pub executable: bool,
    pub uploaded: bool,
    pub sticky: bool,
    pub sticky_lifetime: f64,
    pub sticky_expire_time: common::Time,
    pub signature_required: bool,
    pub is_user_file: bool,
    pub is_project_file: bool,
    pub anonymous_platform_file: bool,
    pub gzip_when_done: bool,
    pub pers_file_xfer: Option<uuid::Uuid>,
    pub result: Option<uuid::Uuid>,
    pub download_urls: Vec<String>,
    pub upload_urls: Vec<String>,
    pub download_gzipped: bool,
    pub xml_signature: String,
    pub file_signature: String,
    pub error_msg: Option<String>,
    pub cert_sigs: Option<cert_sig::CertSigs>,
}
