use crate::lmdb::{ProcessDb, SystemDb};
use crate::sampler::{self, SamplerHandle};
use crate::session;
use crate::system_instance::SystemInstance;
use axum::routing::post;
use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use heed::Env;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub data_fetch: DataFetch,
    pub sampler: Arc<Mutex<Option<SamplerHandle>>>,
}

impl AppState {
    pub fn new(env: Env, process_db: ProcessDb, system_db: SystemDb) -> Self {
        Self {
            data_fetch: DataFetch {
                env,
                process_db,
                system_db,
            },
            sampler: Arc::new(Mutex::new(None)),
        }
    }

    /// Returns false if a sampler is already running.
    pub fn start_sampler(&self) -> bool {
        let mut guard = self.sampler.lock().unwrap();

        if matches!(guard.as_ref(), Some(h) if !h.is_finished()) {
            return false;
        }
        if let Some(dead) = guard.take() {
            dead.stop();
        }

        let d = &self.data_fetch;
        *guard = Some(sampler::start(
            d.env.clone(),
            d.process_db,
            d.system_db,
            session::new_session_id(),
        ));
        true
    }

    pub fn stop_sampler(&self) -> bool {
        let mut guard = self.sampler.lock().unwrap();
        match guard.take() {
            Some(h) => {
                h.stop();
                true
            }
            None => false,
        }
    }
}

#[derive(Clone)]
pub struct DataFetch {
    pub env: Env,
    pub process_db: ProcessDb,
    pub system_db: SystemDb,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/system/latest", get(latest_system))
        .route("/system/status", get(status))
        .route("/system/start", post(start))
        .route("/system/stop", post(stop))
        .with_state(state)
}

async fn stop(State(state): State<AppState>) -> (StatusCode, Json<&'static str>) {
    let stopped = tokio::task::spawn_blocking(move || state.stop_sampler())
        .await
        .unwrap_or(false);

    if stopped {
        (StatusCode::OK, Json("stopped"))
    } else {
        (StatusCode::CONFLICT, Json("not running"))
    }
}

async fn start(State(state): State<AppState>) -> (StatusCode, Json<&'static str>) {
    let started = tokio::task::spawn_blocking(move || state.start_sampler())
        .await
        .unwrap_or(false);

    if started {
        (StatusCode::OK, Json("started"))
    } else {
        (StatusCode::CONFLICT, Json("already running"))
    }
}

async fn latest_system(State(state): State<AppState>) -> Result<Json<SystemInstance>, StatusCode> {
    let s = &state.data_fetch;

    let rtxn = s
        .env
        .read_txn()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match s.system_db.last(&rtxn) {
        Ok(Some((_key, value))) => Ok(Json(value)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn status(State(state): State<AppState>) -> Json<bool> {
    let guard = state.sampler.lock().unwrap();
    Json(matches!(guard.as_ref(), Some(h) if !h.is_finished()))
}
