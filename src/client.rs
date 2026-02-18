use reqwest::Url;

use crate::error::*;
use crate::token::*;
use crate::writer::*;

#[derive(Debug)]
pub struct Client {
    pool: reqwest::Client,
    pool_sync: reqwest::blocking::Client,
    update_uri: Url,
}

impl Client {
    pub fn new(uri: &str) -> Result<Client> {
        Ok(Client {
            pool: reqwest::Client::new(),
            pool_sync: reqwest::blocking::Client::new(),
            update_uri: format!("{}/api/v0/update", uri).parse()?,
        })
    }

    pub fn update_uri(&self) -> &Url {
        &self.update_uri
    }

    pub fn pool(&self) -> &reqwest::Client {
        &self.pool
    }

    pub fn pool_sync(&self) -> &reqwest::blocking::Client {
        &self.pool_sync
    }

    pub fn host_and_maybe_port(&self) -> String {
        let host = self.update_uri.host_str().unwrap_or("localhost");

        self.update_uri
            .port()
            .map(|port| format!("{}:{}", host, port))
            .unwrap_or_else(|| host.to_string())
    }

    pub fn get_writer(&self, token: String) -> Writer<'_> {
        Writer::new(self, Token::new(self, token))
    }
}
