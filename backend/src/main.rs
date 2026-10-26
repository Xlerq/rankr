use std::{env, path::PathBuf};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let directory = env::args_os()
        .nth(1)
        .or_else(|| env::var_os("RANKR_DATA"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data/collected"));
    rankr_import::page::serve(directory).await
}
