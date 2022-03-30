use std::error::Error;

use crate::identity::{AccessResponse, IdentityStore, Outcome};
use ldap3::{LdapConn, Scope, SearchEntry};
use log::info;
use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct LdapSettings {
    pub url: String,
    pub base_dn: String,
    pub bind_dn: String,
    pub bind_password: String,
    pub user_filter: String,
    pub user_name_attr: String,
}

pub struct LdapConnGuard {
    conn: LdapConn,
}

impl LdapConnGuard {
    pub fn new(url: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            conn: LdapConn::new(url)?,
        })
    }

    pub fn as_mut(&mut self) -> &mut LdapConn {
        &mut self.conn
    }
}

impl Drop for LdapConnGuard {
    fn drop(&mut self) {
        let _ = self.conn.unbind();
    }
}

pub struct Ldap {
    settings: LdapSettings,
}

impl Ldap {
    pub fn new(settings: LdapSettings) -> Self {
        Self { settings }
    }
}

impl IdentityStore for Ldap {
    fn access(&mut self, token: &str) -> Result<AccessResponse, Box<dyn Error>> {
        let token = ldap3::ldap_escape(token);
        let filter = &self.settings.user_filter.replace("%t", &token);
        info!("Attempting to initiate LDAP connection...");
        let mut conn_guard = LdapConnGuard::new(&self.settings.url)?;
        let conn = conn_guard.as_mut();
        conn.simple_bind(&self.settings.bind_dn, &self.settings.bind_password)?;
        info!("Reading results...");
        let (results, _) = conn
            .search(
                &self.settings.base_dn,
                Scope::Subtree,
                &filter,
                vec![&self.settings.user_name_attr],
            )?
            .success()?;
        info!("Number of results: {}", results.len());
        Ok(match results.len() {
            0 => AccessResponse {
                outcome: Outcome::Unknown,
                name: None,
            },
            1 => AccessResponse {
                outcome: Outcome::Success,
                name: Some(
                    SearchEntry::construct(results.into_iter().next().unwrap()).attrs
                        [&self.settings.user_name_attr]
                        .join(";"),
                ),
            },
            _ => AccessResponse {
                outcome: Outcome::Revoked,
                name: None,
            },
        })
    }
}
