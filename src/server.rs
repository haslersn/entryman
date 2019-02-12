use crate::history::History;
use crate::history::HistoryEntry;
use crate::identity::IdentityStore;
use crate::identity::Outcome;
use crate::util::Result;
use rocket::config::Config;
use rocket::config::Environment;
use rocket::http::RawStr;
use rocket::request::FromFormValue;
use rocket::State;
use rocket_contrib::json::Json;
use std::sync::Mutex;
use std::time::SystemTime;

pub trait Callback: Send + Sync {
    fn call(&self) -> Result;
}

#[derive(Deserialize)]
pub struct ServerSettings {
    pub mount_point: String,
    pub port: u16,
}

pub struct Context {
    pub identity_store: Box<IdentityStore>,
    pub history: Box<History>,
}

impl<'v> FromFormValue<'v> for Outcome {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> std::result::Result<Outcome, &'v RawStr> {
        match form_value.as_str() {
            "success" => Ok(Outcome::Success),
            "revoked" => Ok(Outcome::Revoked),
            "unknown" => Ok(Outcome::Unknown),
            _ => Err(form_value),
        }
    }
}

pub fn run(settings: ServerSettings, context: Context, callback: Box<Callback>) {
    rocket::custom(
        Config::build(Environment::Staging)
            .address("localhost")
            .port(settings.port)
            .unwrap(),
    )
    .mount(&settings.mount_point, routes![history, access])
    .manage(Mutex::new(context))
    .manage(callback)
    .launch();
}

#[get("/access?<time_min>&<time_max>&<token>&<name>&<outcome>&<only_latest>")]
pub fn history(
    time_min: Option<u64>,
    time_max: Option<u64>,
    token: Option<String>,
    name: Option<String>,
    outcome: Option<Outcome>,
    only_latest: Option<bool>,
    state: State<Mutex<Context>>,
) -> Result<Json<Vec<HistoryEntry>>> {
    let context = state.inner().lock().unwrap();
    let result = context.history.query(
        time_min,
        time_max,
        token.as_ref().map(|x| &**x), // This map turns Option<&String> into Option<&str>
        name.as_ref().map(|x| &**x),
        outcome,
        only_latest.unwrap_or(false),
    )?;
    Ok(Json(result))
}

#[post("/access?<token>")]
pub fn access(
    token: String,
    state: State<Mutex<Context>>,
    callback: State<Box<Callback>>,
) -> Result<Option<String>> {
    let context = &mut state.inner().lock().unwrap();
    let response = context.identity_store.access(&token)?;
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    let name = response.name.clone();
    let outcome = response.outcome;
    context.history.insert(HistoryEntry {
        time,
        token,
        response,
    })?;
    if outcome == Outcome::Success {
        callback.inner().call()?;
    }
    Ok(name)
}
