#[derive(Clone, Debug)]
pub struct Workunit {
    pub name: String,
    pub app_name: String,
    pub command_line: String,
    pub rsc_fpops_est: f64,
    pub rsc_fpops_bound: f64,
    pub rsc_memory_bound: f64,
    pub rsc_disk_bound: f64,
}
