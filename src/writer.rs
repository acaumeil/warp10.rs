use reqwest::{Method, Request, RequestBuilder};

use crate::client::*;
use crate::data::*;
use crate::error::*;
use crate::http_handler;
use crate::response::*;
use crate::token::*;

#[derive(Debug)]
pub struct Writer<'a> {
    client: &'a Client,
    token: Token<'a>,
}

pub fn serialize_data_to_body(data: Vec<Data>) -> String {
    data.iter()
        .map(|d| d.warp10_serialize())
        .fold(String::new(), |acc, cur| {
            if acc.is_empty() {
                cur
            } else {
                (acc + "\n") + &cur
            }
        })
}

impl<'a> Writer<'a> {
    pub fn new(client: &'a Client, token: Token<'a>) -> Self {
        Self { client, token }
    }

    pub async fn post(&self, data: Vec<Data>) -> Result<Warp10Response> {
        let request = self.build_post(data);
        let response = request.send().await?;
        let status = response.status();
        let err = http_handler::extract_header_err(response.headers());
        let payload = response.text().await?;
        http_handler::handle_response(err, status, payload)
    }

    pub fn post_sync(&self, data: Vec<Data>) -> Result<Warp10Response> {
        let request = self.build_post_sync(data);
        let response = request.send()?;
        let status = response.status();
        let err = http_handler::extract_header_err(response.headers());
        let payload = response.text()?;
        http_handler::handle_response(err, status, payload)
    }

    pub fn build_post(&self, data: Vec<Data>) -> RequestBuilder {
        let body = serialize_data_to_body(data);
        let mut base_request = Request::new(Method::POST, self.client.update_uri().clone());
        self.token.set_headers(base_request.headers_mut());
        RequestBuilder::from_parts(self.client.pool().clone(), base_request).body(body)
    }

    pub fn build_post_sync(&self, data: Vec<Data>) -> reqwest::blocking::RequestBuilder {
        let body = serialize_data_to_body(data);
        let mut base_request =
            reqwest::blocking::Request::new(Method::POST, self.client.update_uri().clone());
        self.token.set_headers(base_request.headers_mut());
        reqwest::blocking::RequestBuilder::from_parts(self.client.pool_sync().clone(), base_request)
            .body(body)
    }
}
