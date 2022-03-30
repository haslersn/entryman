use crate::history::{History, HistoryEntry};
use crate::identity::{IdentityStore, Outcome};
use async_trait::async_trait;
use rocket::config::Config;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde_derive::Deserialize;
use std::error::Error;
use std::sync::Mutex;
use std::time::SystemTime;

#[async_trait]
pub trait Callback: Send + Sync {
    async fn call(&self) -> Result<(), Box<dyn Error>>;
}

#[derive(Deserialize)]
pub struct ServerSettings {
    pub mount_point: String,
    pub port: u16,
}

pub struct Context {
    pub identity_store: Box<dyn IdentityStore>,
    pub history: Box<dyn History>,
}

pub async fn run(
    settings: ServerSettings,
    context: Context,
    callback: Box<dyn Callback>,
) -> Result<(), Box<dyn Error>> {
    rocket::custom(Config::figment().merge(("port", settings.port)))
        .mount(&settings.mount_point, routes![history, access])
        .manage(Mutex::new(context))
        .manage(callback)
        .launch()
        .await
        .map_err(Into::into)
}

#[get("/access?<time_min>&<time_max>&<token>&<name>&<outcome>&<only_latest>")]
pub fn history(
    time_min: Option<u64>,
    time_max: Option<u64>,
    token: Option<String>,
    name: Option<String>,
    outcome: Option<Outcome>,
    only_latest: Option<bool>,
    state: &State<Mutex<Context>>,
) -> Result<Json<Vec<HistoryEntry>>, (Status, String)> {
    let context = state.inner().lock().unwrap();
    match context.history.query(
        time_min,
        time_max,
        token.as_ref().map(|x| &**x), // This map turns Option<&String> into Option<&str>
        name.as_ref().map(|x| &**x),
        outcome,
        only_latest.unwrap_or(false),
    ) {
        Err(e) => Err((Status::ServiceUnavailable, e.to_string())),
        Ok(result) => Ok(Json(result)),
    }
}

#[post("/access?<token>")]
pub async fn access(
    token: String,
    state: &State<Mutex<Context>>,
    callback: &State<Box<dyn Callback>>,
) -> Result<Option<String>, (Status, String)> {
    let response = {
        // `context` of type `MutexGuard` cannot be kept accross `.await`, so we encapsulate it in
        // this block.
        let context = &mut state.inner().lock().unwrap();
        let response = context
            .identity_store
            .access(&token)
            .map_err(|e| (Status::Forbidden, e.to_string()))?;
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| (Status::InternalServerError, e.to_string()))?
            .as_secs();
        context
            .history
            .insert(HistoryEntry {
                time,
                token,
                response: response.clone(),
            })
            .map_err(|e| (Status::ServiceUnavailable, e.to_string()))?;
        response
    };
    let name = response.name.clone();
    let outcome = response.outcome;
    if outcome == Outcome::Success {
        callback
            .inner()
            .call()
            .await
            .map_err(|e| (Status::BadGateway, e.to_string()))?;
    }
    Ok(name)
}
