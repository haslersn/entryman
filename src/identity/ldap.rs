use std::error::Error;

use crate::identity::{AccessResponse, IdentityStore, Outcome};
use ldap3::{LdapConnAsync, Scope, SearchEntry};
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
    ldap: ldap3::Ldap,
}

impl LdapConnGuard {
    pub async fn new(url: &str) -> Result<Self, Box<dyn Error>> {
        let (conn, ldap) = LdapConnAsync::new(url).await?;
        ldap3::drive!(conn);
        Ok(Self { ldap })
    }

    pub fn as_mut(&mut self) -> &mut ldap3::Ldap {
        &mut self.ldap
    }
}

impl Drop for LdapConnGuard {
    fn drop(&mut self) {
        let _ = self.ldap.unbind();
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

#[async_trait]
impl IdentityStore for Ldap {
    async fn access(&mut self, token: &str) -> Result<AccessResponse, Box<dyn Error>> {
        let token = ldap3::ldap_escape(token);
        let filter = &self.settings.user_filter.replace("%t", &token);
        info!("Attempting to initiate LDAP connection...");
        let mut ldap_guard = LdapConnGuard::new(&self.settings.url).await?;
        let ldap = ldap_guard.as_mut();
        ldap.simple_bind(&self.settings.bind_dn, &self.settings.bind_password)
            .await?;
        info!("Reading results...");
        let (results, _) = ldap
            .search(
                &self.settings.base_dn,
                Scope::Subtree,
                &filter,
                vec![&self.settings.user_name_attr],
            )
            .await?
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
