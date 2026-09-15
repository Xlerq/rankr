//! Fetching, pure HTML parsing, and JSON persistence are independent modules.
//! A future database adapter can store the model types without invoking the CLI.

pub mod model;
pub mod parser;
pub mod source;
pub mod storage;
