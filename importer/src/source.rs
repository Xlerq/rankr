//! GPW HTTP access and parsing of the current WIG20 constituents.
//!
//! Fetching returns the final response even for HTTP errors. The caller can save
//! the response before checking its status or attempting to parse its contents.

use std::collections::HashSet;

use chrono::Utc;
use reqwest::{Client, RequestBuilder, header};
use scraper::{ElementRef, Html, Selector};
use thiserror::Error;

use crate::{
    http,
    model::{Instrument, MarketPair, RawDocument, Source},
};

const INDEX_ISIN: &str = "PL9999999987";
const INDEX_PAGE: &str = "https://gpwbenchmark.pl/karta-indeksu";
const PORTFOLIO_ENDPOINT: &str = "https://gpwbenchmark.pl/ajaxindex.php";
const FUNDAMENTALS_ENDPOINT: &str = "https://www.gpw.pl/ajaxindex.php";
const TRADINGVIEW_ENDPOINT: &str = "https://scanner.tradingview.com/symbol";
const YAHOO_CHART_ENDPOINT: &str = "https://query1.finance.yahoo.com/v8/finance/chart/";

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("source request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("WIG20 portfolio table with Instrument and Kod ISIN columns was not found")]
    MissingPortfolio,
    #[error("invalid WIG20 constituent in row {row}: {reason}")]
    InvalidConstituent { row: usize, reason: String },
    #[error("expected 20 WIG20 constituents, found {0}")]
    ConstituentCount(usize),
}

pub struct GpwClient {
    client: Client,
}

impl GpwClient {
    pub fn new() -> Result<Self, SourceError> {
        Ok(Self {
            client: http::client()?,
        })
    }

    pub async fn fetch_portfolio(&self) -> Result<RawDocument, SourceError> {
        let page = self
            .fetch(Source::GpwBenchmark, None, || {
                self.client.get(INDEX_PAGE).query(&[("isin", INDEX_ISIN)])
            })
            .await?;

        // Preserve unexpected pages for archiving and diagnosis instead of
        // guessing the AJAX module identifier or losing the received HTML.
        if !(200..300).contains(&page.http_status) {
            return Ok(page);
        }
        let Some(module_id) = extract_module_id(&page.body) else {
            return Ok(page);
        };
        let timestamp = Utc::now().timestamp_millis().to_string();

        self.fetch(Source::GpwBenchmark, None, || {
            self.client
                .post(PORTFOLIO_ENDPOINT)
                .header(header::REFERER, &page.url)
                .query(&[
                    ("action", "GPWIndexes"),
                    ("start", "ajaxPortfolio"),
                    ("format", "html"),
                    ("lang", "PL"),
                    ("isin", INDEX_ISIN),
                    ("cmng_id", module_id),
                    ("time", timestamp.as_str()),
                ])
        })
        .await
    }

    pub async fn fetch_fundamentals(
        &self,
        instrument: &Instrument,
    ) -> Result<RawDocument, SourceError> {
        self.fetch(Source::GpwNotoria, Some(instrument), || {
            self.client.get(FUNDAMENTALS_ENDPOINT).query(&[
                ("action", "GPWListaSp"),
                ("code", instrument.code.as_str()),
                ("format", "html"),
                ("isin", instrument.isin.as_str()),
                ("lang", "PL"),
                ("start", "showNotoria"),
            ])
        })
        .await
    }

    async fn fetch(
        &self,
        source: Source,
        instrument: Option<&Instrument>,
        request: impl Fn() -> RequestBuilder,
    ) -> Result<RawDocument, SourceError> {
        Ok(http::fetch(source, instrument, request).await?)
    }
}

/// Current currency and gold quotes from TradingView's public IDC feed.
pub struct TradingViewClient {
    client: Client,
}

impl TradingViewClient {
    pub fn new() -> Result<Self, SourceError> {
        Ok(Self {
            client: http::client()?,
        })
    }

    pub async fn fetch_price(&self, pair: MarketPair) -> Result<RawDocument, SourceError> {
        let symbol = format!("FX_IDC:{}", pair.code());
        Ok(http::fetch(Source::TradingViewIdc, None, || {
            self.client.get(TRADINGVIEW_ENDPOINT).query(&[
                ("symbol", symbol.as_str()),
                ("fields", "name,close,open,high,low,time,currency"),
            ])
        })
        .await?)
    }
}

/// Yahoo daily company history. Both modes use the same keyless chart endpoint.
pub struct YahooClient {
    client: Client,
    endpoint: String,
}

impl YahooClient {
    pub fn new() -> Result<Self, SourceError> {
        Ok(Self {
            client: http::client()?,
            endpoint: YAHOO_CHART_ENDPOINT.into(),
        })
    }

    pub async fn fetch_history(
        &self,
        instrument: &Instrument,
        symbol: &str,
    ) -> Result<RawDocument, SourceError> {
        self.fetch_daily(instrument, symbol, false).await
    }

    pub async fn fetch_recent_history(
        &self,
        instrument: &Instrument,
        symbol: &str,
    ) -> Result<RawDocument, SourceError> {
        self.fetch_daily(instrument, symbol, true).await
    }

    async fn fetch_daily(
        &self,
        instrument: &Instrument,
        symbol: &str,
        recent: bool,
    ) -> Result<RawDocument, SourceError> {
        let url = format!("{}{symbol}", self.endpoint);
        let end = Utc::now().timestamp().to_string();
        Ok(http::fetch(Source::YahooFinance, Some(instrument), || {
            let request = self.client.get(&url).query(&[
                ("interval", "1d"),
                ("events", "div,splits"),
                ("includeAdjustedClose", "true"),
            ]);
            if recent {
                request.query(&[("range", "1mo")])
            } else {
                // range=max can yield coarser data; explicit bounds retain 1d bars.
                request.query(&[("period1", "0"), ("period2", end.as_str())])
            }
        })
        .await?)
    }
}

/// Parse the current portfolio, rejecting incomplete or ambiguous company lists.
pub fn parse_portfolio(html: &str) -> Result<Vec<Instrument>, SourceError> {
    let document = Html::parse_document(html);
    let tables = Selector::parse("table").expect("valid static selector");
    let headers = Selector::parse("th").expect("valid static selector");
    let rows = Selector::parse("tr").expect("valid static selector");
    let cells = Selector::parse("td").expect("valid static selector");

    let table = document
        .select(&tables)
        .find(|table| {
            let headings: Vec<_> = table.select(&headers).map(element_text).collect();
            headings.first().is_some_and(|value| value == "Instrument")
                && headings.get(1).is_some_and(|value| value == "Kod ISIN")
        })
        .ok_or(SourceError::MissingPortfolio)?;

    let mut instruments = Vec::new();
    let mut known_isins = HashSet::new();
    let mut known_codes = HashSet::new();
    for (row_index, row) in table.select(&rows).enumerate() {
        let values: Vec<_> = row.select(&cells).map(element_text).collect();
        if values.is_empty() {
            continue;
        }

        let invalid = |reason: String| SourceError::InvalidConstituent {
            row: row_index + 1,
            reason,
        };
        if values.len() < 2 {
            return Err(invalid("missing instrument or ISIN column".into()));
        }

        let code = &values[0];
        let isin = &values[1];
        if code.is_empty() {
            return Err(invalid("empty instrument label".into()));
        }
        if !valid_isin(isin) {
            return Err(invalid(format!("invalid ISIN: {isin}")));
        }
        if !known_isins.insert(isin.clone()) {
            return Err(invalid(format!("duplicate ISIN: {isin}")));
        }
        if !known_codes.insert(code.clone()) {
            return Err(invalid(format!("duplicate instrument code: {code}")));
        }

        instruments.push(Instrument {
            isin: isin.clone(),
            code: code.clone(),
            // The source provides an exchange label, not a full legal name.
            name: code.clone(),
        });
    }

    if instruments.len() != 20 {
        return Err(SourceError::ConstituentCount(instruments.len()));
    }

    Ok(instruments)
}

fn extract_module_id(html: &str) -> Option<&str> {
    html.split("cmng_id=").skip(1).find_map(|tail| {
        let end = tail
            .find(|character: char| !character.is_ascii_digit())
            .unwrap_or(tail.len());
        (end > 0).then_some(&tail[..end])
    })
}

fn element_text(element: ElementRef<'_>) -> String {
    element
        .text()
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

fn valid_isin(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 12
        && bytes[..2].iter().all(u8::is_ascii_uppercase)
        && bytes[2..11]
            .iter()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
        && bytes[11].is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        thread,
        time::Duration,
    };

    use super::*;

    fn portfolio(count: usize) -> String {
        let mut html =
            String::from("<table><tr><th>Instrument</th><th>Kod ISIN</th><th>Pakiet</th></tr>");
        for index in 0..count {
            html.push_str(&format!(
                "<tr><td><a> COMPANY{index} </a></td><td>PL{index:010}</td><td>1&nbsp;000</td></tr>"
            ));
        }
        html.push_str("</table>");
        html
    }

    #[test]
    fn reads_twenty_constituents_without_a_local_symbol_map() {
        let html = format!(
            "<table><tr><td>unrelated</td></tr></table>{}",
            portfolio(20)
        );
        let instruments = parse_portfolio(&html).unwrap();
        assert_eq!(instruments.len(), 20);
        assert_eq!(instruments[0].code, "COMPANY0");
        assert_eq!(instruments[0].name, "COMPANY0");
        assert_eq!(instruments[0].isin, "PL0000000000");
    }

    #[test]
    fn rejects_partial_portfolios_and_challenge_pages() {
        assert!(matches!(
            parse_portfolio(&portfolio(19)),
            Err(SourceError::ConstituentCount(19))
        ));
        assert!(matches!(
            parse_portfolio("<h1>Verify your browser</h1>"),
            Err(SourceError::MissingPortfolio)
        ));
    }

    #[test]
    fn rejects_duplicate_identifiers_and_malformed_isins() {
        for (old, new) in [
            ("PL0000000001", "PL0000000000"),
            ("COMPANY1 ", "COMPANY0 "),
            ("PL0000000001", "invalid-isin"),
        ] {
            let html = portfolio(20).replacen(old, new, 1);
            assert!(matches!(
                parse_portfolio(&html),
                Err(SourceError::InvalidConstituent { .. })
            ));
        }
    }

    #[test]
    fn discovers_the_ajax_module_without_a_hardcoded_fallback() {
        assert_eq!(
            extract_module_id("url='ajaxindex.php?cmng_id=1010&time='"),
            Some("1010")
        );
        assert_eq!(extract_module_id("cmng_id=missing"), None);
        assert_eq!(extract_module_id("cmng_id="), None);
        assert_eq!(extract_module_id("<h1>Unexpected page</h1>"), None);
    }

    fn serve(responses: Vec<&'static str>) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let handle = thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut request = BufReader::new(&stream);
                loop {
                    let mut line = String::new();
                    assert!(request.read_line(&mut line).unwrap() > 0);
                    if line == "\r\n" {
                        break;
                    }
                }
                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        (url, handle)
    }

    #[tokio::test]
    async fn preserves_http_error_responses_for_archiving() {
        let (url, server) = serve(vec![
            "HTTP/1.1 404 Not Found\r\nContent-Length: 7\r\nConnection: close\r\n\r\nmissing",
        ]);
        let source = GpwClient {
            client: Client::new(),
        };
        let raw = source
            .fetch(Source::GpwNotoria, None, || source.client.get(&url))
            .await
            .unwrap();
        server.join().unwrap();
        assert_eq!(raw.http_status, 404);
        assert_eq!(raw.body, "missing");
        assert_eq!(raw.url, url);
    }

    #[tokio::test]
    async fn retries_transient_server_errors() {
        let (url, server) = serve(vec![
            "HTTP/1.1 503 Service Unavailable\r\nContent-Length: 4\r\nConnection: close\r\n\r\nbusy",
            "HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nready",
        ]);
        let source = GpwClient {
            client: Client::new(),
        };
        let raw = source
            .fetch(Source::GpwNotoria, None, || source.client.get(&url))
            .await
            .unwrap();
        server.join().unwrap();
        assert_eq!(raw.http_status, 200);
        assert_eq!(raw.body, "ready");
    }

    fn serve_yahoo(status: u16, body: &str) -> (String, thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!(
            "http://{}/v8/finance/chart/",
            listener.local_addr().unwrap()
        );
        let response = format!(
            "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let handle = thread::spawn(move || {
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(std::time::Instant::now() < deadline, "request timed out");
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("local server failed: {error}"),
                }
            };
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = BufReader::new(&stream);
            let mut first_line = String::new();
            request.read_line(&mut first_line).unwrap();
            loop {
                let mut line = String::new();
                assert!(request.read_line(&mut line).unwrap() > 0);
                if line == "\r\n" {
                    break;
                }
            }
            stream.write_all(response.as_bytes()).unwrap();
            first_line
        });
        (url, handle)
    }

    #[tokio::test]
    async fn yahoo_requests_daily_history_without_authentication_or_aggregation() {
        use crate::{
            history::{YahooSymbolMap, parse_history_document},
            storage::{archive_raw, read_raw},
        };
        let body = include_str!("../tests/fixtures/yahoo_daily.json");
        for recent in [false, true] {
            let (endpoint, server) = serve_yahoo(200, body);
            let client = YahooClient {
                client: Client::builder().no_proxy().build().unwrap(),
                endpoint,
            };
            let instrument = YahooSymbolMap::bundled()
                .unwrap()
                .instrument("KGHM")
                .unwrap();
            let raw = if recent {
                client.fetch_recent_history(&instrument, "KGH.WA").await
            } else {
                client.fetch_history(&instrument, "KGH.WA").await
            }
            .unwrap();
            let request = server.join().unwrap();
            let requested = reqwest::Url::parse(&format!(
                "http://localhost{}",
                request.split_whitespace().nth(1).unwrap()
            ))
            .unwrap();
            assert_eq!(requested.path(), "/v8/finance/chart/KGH.WA");
            let query: std::collections::HashMap<_, _> = requested.query_pairs().collect();
            assert_eq!(query.get("interval").unwrap(), "1d");
            assert_eq!(query.get("events").unwrap(), "div,splits");
            assert_eq!(query.get("includeAdjustedClose").unwrap(), "true");
            if recent {
                assert_eq!(query.get("range").unwrap(), "1mo");
                assert_eq!(query.len(), 4);
            } else {
                assert_eq!(query.get("period1").unwrap(), "0");
                assert!(query.get("period2").unwrap().parse::<i64>().unwrap() > 0);
                assert_eq!(query.len(), 5);
            }
            let directory = tempfile::tempdir().unwrap();
            let archived = archive_raw(directory.path(), &raw).unwrap();
            let saved = read_raw(&archived.raw_path()).unwrap();
            assert_eq!(saved.source, Source::YahooFinance);
            assert_eq!(saved.body, body);
            assert_eq!(parse_history_document(&saved).unwrap().candles.len(), 3);
        }
    }

    #[tokio::test]
    async fn yahoo_http_errors_remain_available_for_archiving() {
        let (endpoint, server) = serve_yahoo(403, "Forbidden");
        let client = YahooClient {
            client: Client::builder().no_proxy().build().unwrap(),
            endpoint,
        };
        let instrument = crate::history::YahooSymbolMap::bundled()
            .unwrap()
            .instrument("KGHM")
            .unwrap();
        let raw = client.fetch_history(&instrument, "KGH.WA").await.unwrap();
        server.join().unwrap();
        assert_eq!(raw.http_status, 403);
        assert_eq!(raw.body, "Forbidden");
        assert!(matches!(
            crate::history::parse_history_document(&raw),
            Err(crate::history::HistoryError::HttpStatus(403))
        ));
    }
}
