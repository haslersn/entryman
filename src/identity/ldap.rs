use crate::identity::AccessResponse;
use crate::identity::IdentityStore;
use crate::identity::Outcome;
use crate::util::Result;
use ldap3::LdapConn;
use ldap3::Scope;
use ldap3::SearchEntry;
use std::slice::SliceConcatExt;

#[derive(Deserialize)]
pub struct LdapSettings {
    pub url: String,
    pub base_dn: String,
    pub bind_dn: String,
    pub bind_password: String,
    pub user_filter: String,
    pub user_name_attr: String,
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
    fn access(&mut self, token: &str) -> Result<AccessResponse> {
        let token = ldap3::ldap_escape(token);
        let filter = &self.settings.user_filter.replace("%t", &token);
        info!("Attempting to initiate LDAP connection...");
        let conn = LdapConn::new(&self.settings.url)?;
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
        conn.unbind();
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
