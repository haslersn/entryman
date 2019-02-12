#![feature(
    proc_macro_hygiene,
    decl_macro,
    slice_concat_ext,
    unboxed_closures,
    fn_traits
)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

pub mod client;
pub mod history;
pub mod identity;
pub mod server;
pub mod util;

use client::Client;
use client::ClientSettings;
use env_logger::Builder;
use env_logger::Env;
use history::json_history::JsonHistory;
use history::json_history::JsonHistorySettings;
use identity::ldap::Ldap;
use identity::ldap::LdapSettings;
use server::Callback;
use server::Context;
use server::ServerSettings;
use util::Result;

impl Callback for Client {
    fn call(&self) -> Result {
        self.access()
    }
}

#[derive(Deserialize)]
struct Config {
    ldap: LdapSettings,
    json_history: JsonHistorySettings,
    server: ServerSettings,
    client: ClientSettings,
}

fn main() -> Result {
    // initialize logger w/ log level "info"
    Builder::from_env(Env::new().default_filter_or("info")).init();

    // configure and start web server
    let conf = read_config()?;
    let context = Context {
        identity_store: Box::new(Ldap::new(conf.ldap)),
        history: Box::new(JsonHistory::new(conf.json_history)?),
    };
    let client = Box::new(Client::new(conf.client));
    server::run(conf.server, context, client);
    Ok(())
}

fn read_config() -> Result<Config> {
    let conf_str = std::fs::read_to_string("entman.toml")?;
    toml::from_str(&conf_str).map_err(From::from)
}
