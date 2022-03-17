#[macro_use]
extern crate rocket;

pub mod client;
pub mod history;
pub mod identity;
pub mod server;

use std::error::Error;

use client::{Client, ClientSettings};
use env_logger::{Builder, Env};
use history::json_history::{JsonHistory, JsonHistorySettings};
use identity::ldap::{Ldap, LdapSettings};
use identity::json::{Json, JsonIdentitySettings};
use serde_derive::Deserialize;
use server::{Callback, Context, ServerSettings};
#[async_trait]
impl Callback for Client {
    async fn call(&self) -> Result<(), Box<dyn Error>> {
        self.access().await
    }
}

#[derive(Deserialize)]
struct MainSettings {
    identity_store: IdentityStoreType
}

#[derive(Deserialize)]
enum IdentityStoreType {
  Ldap,
  Json,
}

#[derive(Deserialize)]
struct Config {
    main: MainSettings,
    ldap: LdapSettings,
    json_identity: JsonIdentitySettings,
    json_history: JsonHistorySettings,
    server: ServerSettings,
    client: ClientSettings,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // initialize logger w/ log level "info"
    Builder::from_env(Env::new().default_filter_or("info")).init();

    // configure and start web server
    let conf = read_config()?;
    let context = Context {
        identity_store: match conf.main.identity_store {
            IdentityStoreType::Ldap => Box::new(Ldap::new(conf.ldap)),
            IdentityStoreType::Json => Box::new(Json::new(conf.json_identity)),
        },
        history: Box::new(JsonHistory::new(conf.json_history)?),
    };
    let client = Box::new(Client::new(conf.client));
    server::run(conf.server, context, client).await
}

fn read_config() -> Result<Config, Box<dyn Error>> {
    let conf_str = std::fs::read_to_string("entman.toml")?;
    toml::from_str(&conf_str).map_err(From::from)
}
