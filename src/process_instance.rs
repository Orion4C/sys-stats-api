use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sysinfo::Process;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessInstance {
    pub pname: String,
    pub pid: u32,
    pub usage_cpu: f32,
    pub usage_mem: u64,
    pub usage_diskr: u64,
    pub usage_diskw: u64,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub created_at: DateTime<Utc>,
}

impl ProcessInstance {
    pub fn new(process: &Process, time: DateTime<Utc>) -> Self {
        let disk = process.disk_usage();
        Self {
            pname: process.name().to_string_lossy().into_owned(),
            pid: process.pid().as_u32(),
            usage_cpu: process.cpu_usage(),
            usage_mem: process.memory(),
            usage_diskr: disk.read_bytes,
            usage_diskw: disk.written_bytes,
            created_at: time,
        }
    }
}
