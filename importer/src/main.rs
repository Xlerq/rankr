use std::path::PathBuf;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let directory = PathBuf::from("data/collected");
    eprintln!("Refreshing WIG20 data in {}", directory.display());
    if let Err(error) = rankr::data_refresh::refresh(&directory).await {
        eprintln!("Refresh error: {error:#}");
    }
    match rankr::ranking::archive_snapshot(&directory, chrono::Utc::now()) {
        Ok(Some(path)) => eprintln!("Ranking saved: {}", path.display()),
        Ok(None) => eprintln!("No ranking data to archive"),
        Err(error) => eprintln!("Ranking history error: {error:#}"),
    }
    // Await the whole refresh before opening the listening socket.
    rankr::page::serve(directory).await
}
