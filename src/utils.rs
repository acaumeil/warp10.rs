use reqwest::RequestBuilder;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};

pub const API_ENDPOINT_UPDATE: &str = "/api/v0/update";

pub const HTTP_HEADER_ERROR_MESSAGE: HeaderName = HeaderName::from_static("x-warp10-error-message");
pub const HTTP_HEADER_TOKEN: HeaderName = HeaderName::from_static("x-warp10-token");
pub const HTTP_HEADER_UPDATE_TOKEN: HeaderName = HeaderName::from_static("x-warp10-token");

#[macro_export]
macro_rules! http_header {
    ($( ( $name:ident , $value:literal ) ),+ $(,)?) => {
        $(
            pub const $name: http::header::HeaderName =
                http::header::HeaderName::from_static($value);
        )+
    };
}

pub fn set_ingress_headers(
    request: RequestBuilder,
    write_token: &str,
    body_compression: bool,
) -> RequestBuilder {
    let mut headers = HeaderMap::new();
    set_token_ingress_header(write_token, &mut headers);
    if body_compression {
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/gzip"));
    }
    request.headers(headers)
}

pub fn set_token_ingress_header(write_token: &str, headers: &mut HeaderMap) {
    set_token_header(write_token, headers, Some(HTTP_HEADER_UPDATE_TOKEN));
}

pub fn set_token_header(token: &str, headers: &mut HeaderMap, token_header: Option<HeaderName>) {
    if let Ok(token) = HeaderValue::from_str(token) {
        headers.insert(token_header.unwrap_or(HTTP_HEADER_TOKEN), token);
    }
}

pub fn extract_header_err(headers: &HeaderMap<HeaderValue>) -> Option<String> {
    // Extract the error header from the Warp 10 response, the header is defined here
    // https://github.com/senx/warp10-platform/blob/master/warp10/src/main/java/io/warp10/continuum/store/Constants.java#L191
    headers
        .get(HTTP_HEADER_ERROR_MESSAGE)
        .map(|hv| hv.as_bytes().to_vec())
        .and_then(|buf| String::from_utf8(buf).ok())
}
