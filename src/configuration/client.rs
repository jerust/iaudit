use std::time::Duration;

use reqwest::{Client, Error};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ClientSettings {
    pub timeout: u64,
}

impl ClientSettings {
    pub fn get_proxy_client(&self) -> Result<Client, Error> {
        Client::builder()
            .timeout(Duration::from_secs(self.timeout))
            .build()
    }
}
