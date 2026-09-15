//! Fetching, pure HTML parsing, and JSON persistence are independent modules.
//! A future database adapter can store the model types without invoking the CLI.

mod http;
pub mod model;
pub mod parser;
pub mod prices;
pub mod source;
pub mod storage;
