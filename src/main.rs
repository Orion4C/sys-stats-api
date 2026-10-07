mod api;
mod lmdb;
mod process_instance;
mod sampler;
mod session;
mod system_instance;

use crate::api::AppState;
use chrono::Utc;

const MAP_SIZE: usize = 8 * 1024 * 1024 * 1024;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Utc::now().format("data/%Y-%m-%d.mdb").to_string();

    let env = lmdb::open_env(path, MAP_SIZE, 4)?;
    let process_db = lmdb::open_or_create_db::<_, _>(&env, "processes")?;
    let system_db = lmdb::open_or_create_db::<_, _>(&env, "system")?;

    let state = AppState::new(env, process_db, system_db);

    let app = api::router(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;

    tokio::task::spawn_blocking(move || state.stop_sampler()).await?;

    Ok(())
}

//# start
// curl -X POST http://127.0.0.1:3000/system/start

// # stop
// curl -X POST http://127.0.0.1:3000/system/stop

// # check if it's running
// curl http://127.0.0.1:3000/system/status

// # latest system sample
// curl http://127.0.0.1:3000/system/latest
