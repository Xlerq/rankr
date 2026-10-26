//! HTTP access for the three data sources read by the ranking.

use chrono::{DateTime, Days, NaiveDate, Utc};
use reqwest::{Client, RequestBuilder};

use crate::{
    http,
    model::{Instrument, RawDocument, Source},
};

const FUNDAMENTALS_ENDPOINT: &str = "https://www.gpw.pl/ajaxindex.php";
const YAHOO_CHART_ENDPOINT: &str = "https://query1.finance.yahoo.com/v8/finance/chart/";
const KNF_ENDPOINT: &str = "https://rss.knf.gov.pl/rss_pub/JSON";

pub struct GpwClient {
    client: Client,
}

impl GpwClient {
    pub fn new() -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: http::client()?,
        })
    }

    pub async fn fetch_fundamentals(
        &self,
        instrument: &Instrument,
    ) -> Result<RawDocument, reqwest::Error> {
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
    ) -> Result<RawDocument, reqwest::Error> {
        http::fetch(source, instrument, request).await
    }
}

/// Fetch the full daily series once, then only the dates after the last stored candle.
pub struct YahooClient {
    client: Client,
    endpoint: String,
}

impl YahooClient {
    pub fn new() -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: http::client()?,
            endpoint: YAHOO_CHART_ENDPOINT.into(),
        })
    }

    pub async fn fetch_history(
        &self,
        instrument: &Instrument,
        symbol: &str,
        last_date: Option<NaiveDate>,
        now: DateTime<Utc>,
    ) -> Result<RawDocument, reqwest::Error> {
        let url = format!("{}{symbol}", self.endpoint);
        let start = last_date
            .map(|date| {
                (date + Days::new(1))
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
                    .timestamp()
            })
            .unwrap_or(0);
        // Explicit bounds preserve daily candles even for the initial full history.
        http::fetch(Source::YahooFinance, Some(instrument), || {
            self.client
                .get(&url)
                .query(&[
                    ("interval", "1d"),
                    ("events", "div,splits"),
                    ("includeAdjustedClose", "true"),
                ])
                .query(&[("period1", start), ("period2", now.timestamp())])
        })
        .await
    }
}

/// The current register uses an undocumented endpoint of the public KNF website.
pub struct KnfClient {
    client: Client,
}

impl KnfClient {
    pub fn new() -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: http::client()?,
        })
    }

    pub async fn fetch_shorts(
        &self,
        instrument: &Instrument,
        offset: usize,
    ) -> Result<RawDocument, reqwest::Error> {
        http::fetch(Source::KnfShorts, Some(instrument), || {
            self.request_shorts(&instrument.isin, offset)
        })
        .await
    }

    fn request_shorts(&self, isin: &str, offset: usize) -> RequestBuilder {
        let request = serde_json::json!({
            "cmd": "get",
            "method": "Default",
            "language": "pl",
            "limit": 100,
            "offset": offset,
            "search": [{"field": "ISIN", "type": "text", "operator": "contains", "value": isin}],
            "sort": [{"field": "HOLDER_FULL_NAME", "direction": "asc"}],
            "searchLogic": "AND",
            "searchValue": ""
        });
        self.client
            .post(KNF_ENDPOINT)
            .form(&[("request", request.to_string())])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header;
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        thread,
        time::Duration,
    };

    #[test]
    fn knf_request_posts_a_form_for_the_current_register_and_requested_isin() {
        let client = KnfClient::new().unwrap();
        let request = client.request_shorts("PLBUDMX00013", 100).build().unwrap();
        assert_eq!(request.method(), reqwest::Method::POST);
        assert_eq!(
            request.url().as_str(),
            "https://rss.knf.gov.pl/rss_pub/JSON"
        );
        assert_eq!(
            request.headers()[header::CONTENT_TYPE],
            "application/x-www-form-urlencoded"
        );
        let body = std::str::from_utf8(request.body().unwrap().as_bytes().unwrap()).unwrap();
        let mut encoded_form = reqwest::Url::parse("https://example.invalid").unwrap();
        encoded_form.set_query(Some(body));
        let fields: Vec<_> = encoded_form.query_pairs().collect();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].0, "request");
        let json: serde_json::Value = serde_json::from_str(&fields[0].1).unwrap();
        assert_eq!(json["cmd"], "get");
        assert_eq!(json["method"], "Default");
        assert_eq!(json["language"], "pl");
        assert_eq!(json["limit"], 100);
        assert_eq!(json["offset"], 100);
        assert_eq!(
            json["search"],
            serde_json::json!([
                {"field": "ISIN", "type": "text", "operator": "contains", "value": "PLBUDMX00013"}
            ])
        );
        assert_eq!(
            json["sort"],
            serde_json::json!([
                {"field": "HOLDER_FULL_NAME", "direction": "asc"}
            ])
        );
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
    async fn history_requests_only_later_dates_and_appends_without_duplicates() {
        use crate::{
            data_collect,
            history::{YahooSymbolMap, parse_history_document},
            model::DailyPriceHistory,
        };
        use chrono::TimeZone;
        use serde_json::{Value, json};

        let now = Utc.with_ymd_and_hms(2026, 10, 23, 12, 0, 0).unwrap();
        let symbols = YahooSymbolMap::bundled().unwrap();
        let instrument = symbols.instrument("KGHM").unwrap();
        let mut body: Value =
            serde_json::from_str(include_str!("../tests/fixtures/yahoo_daily.json")).unwrap();
        body["chart"]["result"][0]["timestamp"] = json!([20, 21, 22].map(|day| {
            Utc.with_ymd_and_hms(2026, 10, day, 8, 0, 0)
                .unwrap()
                .timestamp()
        }));
        let raw = RawDocument {
            instrument: Some(instrument.clone()),
            source: Source::YahooFinance,
            fetched_at: now,
            url: "https://query1.finance.yahoo.com/v8/finance/chart/KGH.WA?interval=1d".into(),
            http_status: 200,
            body: body.to_string(),
        };
        let mut initial = parse_history_document(&raw).unwrap();
        initial.candles.truncate(1);
        initial.fetched_at = Utc.with_ymd_and_hms(2026, 10, 20, 19, 0, 0).unwrap();
        let directory = tempfile::tempdir().unwrap();
        let path = directory
            .path()
            .join("2026-10-20_KGHM_history/history.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_vec(&initial).unwrap()).unwrap();

        let (endpoint, server) = serve_yahoo(200, &raw.body);
        let client = YahooClient {
            client: Client::builder().no_proxy().build().unwrap(),
            endpoint,
        };
        let saved = data_collect::latest_histories(std::slice::from_ref(&path)).remove("KGHM");
        data_collect::collect_history(&client, &symbols, &instrument, directory.path(), saved, now)
            .await
            .unwrap();
        let request = server.join().unwrap();
        let requested = reqwest::Url::parse(&format!(
            "http://localhost{}",
            request.split_whitespace().nth(1).unwrap()
        ))
        .unwrap();
        let query: std::collections::HashMap<_, _> = requested.query_pairs().collect();
        assert_eq!(
            query["period1"],
            Utc.with_ymd_and_hms(2026, 10, 21, 0, 0, 0)
                .unwrap()
                .timestamp()
                .to_string()
        );
        assert_eq!(query["period2"], now.timestamp().to_string());
        assert_eq!(query["interval"], "1d");
        assert!(!query.contains_key("range"));
        let saved_bytes = std::fs::read(&path).unwrap();
        let saved: DailyPriceHistory = serde_json::from_slice(&saved_bytes).unwrap();
        assert_eq!(
            saved
                .candles
                .iter()
                .map(|candle| candle.date.to_string())
                .collect::<Vec<_>>(),
            ["2026-10-20", "2026-10-21", "2026-10-22"]
        );
        assert_eq!(saved.candles[0], initial.candles[0]);

        // An empty update must not rewrite history.json or create another raw archive.
        let observations = std::fs::read_dir(directory.path()).unwrap().count();
        body["chart"]["result"][0]["timestamp"] = Value::Null;
        body["chart"]["result"][0]["indicators"] = json!({"quote": [{}]});
        let (endpoint, server) = serve_yahoo(200, &body.to_string());
        let client = YahooClient {
            client: Client::builder().no_proxy().build().unwrap(),
            endpoint,
        };
        let history = data_collect::SavedHistory {
            path: path.clone(),
            history: saved,
        };
        data_collect::collect_history(
            &client,
            &symbols,
            &instrument,
            directory.path(),
            Some(history),
            now,
        )
        .await
        .unwrap();
        server.join().unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), saved_bytes);
        assert_eq!(
            std::fs::read_dir(directory.path()).unwrap().count(),
            observations
        );
    }
}
