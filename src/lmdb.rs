use std::fs;
use std::path::Path;

use crate::{process_instance::ProcessInstance, system_instance::SystemInstance};
use heed::{
    Database, Env, EnvOpenOptions,
    types::{Bytes, SerdeBincode},
};

pub type ProcessDb = Database<Bytes, SerdeBincode<ProcessInstance>>;
pub type SystemDb = Database<Bytes, SerdeBincode<SystemInstance>>;

pub fn open_env(dir: impl AsRef<Path>, map_size: usize, max_dbs: u32) -> heed::Result<Env> {
    fs::create_dir_all(dir.as_ref())?;

    let env = unsafe {
        EnvOpenOptions::new()
            .map_size(map_size)
            .max_dbs(max_dbs)
            .open(dir)?
    };
    Ok(env)
}

pub fn open_or_create_db<K: 'static, V: 'static>(
    env: &Env,
    name: &str,
) -> heed::Result<Database<K, V>> {
    let mut wtxn = env.write_txn()?;
    let db = env.create_database(&mut wtxn, Some(name))?;
    wtxn.commit()?;
    Ok(db)
}
