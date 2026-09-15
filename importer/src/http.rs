//! Shared transport. Responses remain available for archiving on HTTP errors.

use std::time::Duration;

use chrono::Utc;
use reqwest::{Client, RequestBuilder, StatusCode, header};

use crate::model::{Instrument, RawDocument, Source};

const MAX_ATTEMPTS: u32 = 3;

pub(crate) fn client() -> Result<Client, reqwest::Error> {
    let mut headers = header::HeaderMap::new();
    headers.insert(header::ACCEPT, header::HeaderValue::from_static("*/*"));
    headers.insert(
        header::ACCEPT_LANGUAGE,
        header::HeaderValue::from_static("pl,en;q=0.8"),
    );
    Client::builder()
        .user_agent(concat!("rankr-import/", env!("CARGO_PKG_VERSION")))
        .default_headers(headers)
        .https_only(true)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(20))
        .retry(reqwest::retry::never())
        .build()
}

pub(crate) async fn fetch(
    source: Source,
    instrument: Option<&Instrument>,
    request: impl Fn() -> RequestBuilder,
) -> Result<RawDocument, reqwest::Error> {
    for attempt in 1..=MAX_ATTEMPTS {
        let result = async {
            let response = request().send().await?;
            let status = response.status();
            let url = response.url().to_string();
            let body = response.text().await?;
            Ok::<_, reqwest::Error>((status, url, body))
        }
        .await;

        match result {
            Ok((status, _, _))
                if attempt < MAX_ATTEMPTS
                    && (status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()) => {}
            Ok((status, url, body)) => {
                return Ok(RawDocument {
                    source,
                    instrument: instrument.cloned(),
                    fetched_at: Utc::now(),
                    url,
                    http_status: status.as_u16(),
                    body,
                });
            }
            Err(error)
                if attempt < MAX_ATTEMPTS
                    && (error.is_timeout()
                        || error.is_connect()
                        || error.is_request()
                        || error.is_body()) => {}
            Err(error) => return Err(error),
        }

        tokio::time::sleep(Duration::from_secs(u64::from(attempt))).await;
    }

    unreachable!("the final attempt always returns its response or error")
}
