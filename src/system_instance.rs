use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInstance {
    pub cpu_count: usize,
    pub usage_cpu: f32,
    pub usage_mem: u64,
    pub total_mem: u64,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub created_at: DateTime<Utc>,
}

impl SystemInstance {
    pub fn new(sys: &System, time: DateTime<Utc>) -> Self {
        Self {
            cpu_count: sys.cpus().len(),
            usage_cpu: sys.global_cpu_usage(),
            usage_mem: sys.used_memory(),
            total_mem: sys.total_memory(),
            created_at: time,
        }
    }
}
