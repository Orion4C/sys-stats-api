use std::{
    sync::mpsc::{self, RecvTimeoutError},
    thread::{self, JoinHandle},
    time::Duration,
};

use chrono::Utc;
use sysinfo::{MINIMUM_CPU_UPDATE_INTERVAL, ProcessesToUpdate, System};
use uuid::Uuid;

use crate::lmdb::{ProcessDb, SystemDb};
use crate::process_instance::ProcessInstance;
use crate::session;
use crate::system_instance::SystemInstance;

const SAMPLE_INTERVAL: Duration = Duration::from_secs(1);

pub struct SamplerHandle {
    stop_tx: mpsc::Sender<()>,
    handle: JoinHandle<()>,
}

impl SamplerHandle {
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    pub fn stop(self) {
        let _ = self.stop_tx.send(());
        let _ = self.handle.join();
    }
}

pub fn start(
    env: heed::Env,
    process_db: ProcessDb,
    system_db: SystemDb,
    session_id: Uuid,
) -> SamplerHandle {
    let (stop_tx, stop_rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let mut sys = System::new_all();

        let mut sys_seq: u64 = 0;
        let mut proc_seq: u64 = 0;

        thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);

        loop {
            match stop_rx.recv_timeout(SAMPLE_INTERVAL) {
                Err(RecvTimeoutError::Timeout) => {
                    sys.refresh_cpu_usage();
                    sys.refresh_memory();
                    sys.refresh_processes(ProcessesToUpdate::All, true);
                    let now = Utc::now();

                    let mut wtxn = env.write_txn().unwrap();

                    let instance = SystemInstance::new(&sys, now);
                    let key = session::session_key(session_id, sys_seq);
                    system_db.put(&mut wtxn, &key[..], &instance).unwrap();

                    for process in sys.processes().values() {
                        let instance = ProcessInstance::new(process, now);
                        let key = session::session_key(session_id, proc_seq);
                        process_db.put(&mut wtxn, &key[..], &instance).unwrap();
                        proc_seq += 1;
                    }

                    wtxn.commit().unwrap();
                    sys_seq += 1;
                }
                _ => break,
            }
            thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
        }
    });

    SamplerHandle { stop_tx, handle }
}
