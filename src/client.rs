use std::fmt::Debug;
use std::io::{Error, ErrorKind};

use crate::error::{ClientInitializationError, ClientRequestError};
use crate::gts::{Gts, Warp10Serializable};
use crate::utils::{API_ENDPOINT_UPDATE, set_ingress_headers};

use reqwest::Url;

#[derive(Debug, Clone)]
pub struct Client {
    pub client: reqwest::Client,
    pub base_url: Url,
    pub read_token: Option<String>,
    pub write_token: Option<String>,
}

impl Client {
    pub fn new(base_url: &str) -> Result<Client, ClientInitializationError> {
        Ok(Client {
            client: reqwest::ClientBuilder::new().build()?,
            base_url: { Url::parse(base_url)? },
            read_token: None,
            write_token: None,
        })
    }

    pub fn set_read_token(&mut self, token: String) {
        self.read_token = Some(token);
    }

    pub fn set_write_token(&mut self, token: String) {
        self.write_token = Some(token);
    }

    pub fn ingress_url(&self) -> Result<Url, url::ParseError> {
        self.base_url.join(API_ENDPOINT_UPDATE)
    }

    pub async fn ingress_gts(
        &self,
        series: Vec<Gts>,
        body_compression: bool,
    ) -> Result<(), ClientRequestError> {
        self.ingress_gts_as_str(series.warp10_serialize(), body_compression)
            .await
    }

    pub async fn ingress_gts_as_str(
        &self,
        series: String,
        body_compression: bool,
    ) -> Result<(), ClientRequestError> {
        let mut request = self.client.post(self.ingress_url()?);
        request = set_ingress_headers(
            request,
            self.write_token
                .clone()
                .ok_or(ClientRequestError::TokenUnset(Error::new(
                    ErrorKind::InvalidInput,
                    "Write Token is unset",
                )))?
                .as_ref(),
            body_compression,
        );
        request.body(series).send().await?;
        Ok(())
    }
}
