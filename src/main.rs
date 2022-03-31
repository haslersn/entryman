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
use serde_derive::Deserialize;
use server::{Callback, Context, ServerSettings};

#[async_trait]
impl Callback for Client {
    async fn call(&self) -> Result<(), Box<dyn Error>> {
        self.access().await
    }
}

#[derive(Deserialize)]
struct Config {
    ldap: LdapSettings,
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
        identity_store: Box::new(Ldap::new(conf.ldap)),
        history: Box::new(JsonHistory::new(conf.json_history)?),
    };
    let client = Box::new(Client::new(conf.client));
    server::run(conf.server, context, client).await
}

fn read_config() -> Result<Config, Box<dyn Error>> {
    let conf_str = std::fs::read_to_string("entman.toml")?;
    toml::from_str(&conf_str).map_err(From::from)
}
