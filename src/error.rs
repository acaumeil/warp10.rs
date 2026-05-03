use std::{
    fmt::{self},
    io,
};

#[derive(Debug)]
pub enum ClientInitializationError {
    UrlParseError(url::ParseError),
    ClientBuildingError(reqwest::Error),
}

impl From<url::ParseError> for ClientInitializationError {
    fn from(value: url::ParseError) -> Self {
        Self::UrlParseError(value)
    }
}

impl From<reqwest::Error> for ClientInitializationError {
    fn from(value: reqwest::Error) -> Self {
        Self::ClientBuildingError(value)
    }
}

impl fmt::Display for ClientInitializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientInitializationError::UrlParseError(err) => {
                write!(f, "Client initialization error: could not parse URL: {err}")
            }
            ClientInitializationError::ClientBuildingError(err) => write!(
                f,
                "Client initialization error: could not build HTTP client: {err}"
            ),
        }
    }
}

#[derive(Debug)]
pub enum ClientRequestError {
    UrlParseError(url::ParseError),
    TokenUnset(io::Error),
    ClientSendError(reqwest::Error),
}

impl From<url::ParseError> for ClientRequestError {
    fn from(value: url::ParseError) -> Self {
        Self::UrlParseError(value)
    }
}

impl From<io::Error> for ClientRequestError {
    fn from(value: io::Error) -> Self {
        Self::TokenUnset(value)
    }
}

impl From<reqwest::Error> for ClientRequestError {
    fn from(value: reqwest::Error) -> Self {
        Self::ClientSendError(value)
    }
}

impl fmt::Display for ClientRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientRequestError::UrlParseError(err) => {
                write!(f, "Client request error: could not parse URL: {err}")
            }
            ClientRequestError::TokenUnset(err) => write!(f, "Client request error: {err}"),
            ClientRequestError::ClientSendError(err) => {
                write!(f, "Client request error: request failed: {err}")
            }
        }
    }
}
