use crate::util::Result;

#[derive(Deserialize)]
pub struct ClientSettings {
    endpoint: String,
}

pub struct Client {
    settings: ClientSettings,
    rest_client: reqwest::Client,
}

impl Client {
    pub fn new(settings: ClientSettings) -> Client {
        Client {
            settings,
            rest_client: reqwest::Client::new(),
        }
    }

    pub fn access(&self) -> Result {
        let url = &self.settings.endpoint;
        self.rest_client.post(url).send()?;
        Ok(())
    }
}
