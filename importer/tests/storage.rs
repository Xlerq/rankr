use chrono::{TimeZone, Utc};
use rankr::{
    model::{Instrument, RawDocument, Source},
    storage::archive_raw,
};

fn read_raw(path: &std::path::Path) -> serde_json::Result<RawDocument> {
    serde_json::from_slice(&std::fs::read(path).unwrap())
}

fn raw() -> RawDocument {
    RawDocument {
        source: Source::GpwNotoria,
        instrument: Some(Instrument {
            isin: "PLKGHM000017".into(),
            code: "KGHM".into(),
            name: "KGHM".into(),
        }),
        fetched_at: Utc.with_ymd_and_hms(2026, 9, 14, 18, 0, 0).unwrap(),
        url: "https://example.invalid/fundamentals".into(),
        http_status: 200,
        body: "<h3>Unrecognized report</h3>".into(),
    }
}

#[test]
fn raw_survives_parser_failure_and_duplicate_timestamps() {
    let root = tempfile::tempdir().unwrap();
    let document = raw();
    let first = archive_raw(root.path(), &document).unwrap();
    first.save_error("financial table missing").unwrap();
    let second = archive_raw(root.path(), &document).unwrap();

    assert_eq!(
        first.raw_path(),
        root.path().join("2026-09-14_KGHM_fundamentals/raw.json")
    );
    assert_eq!(
        second.raw_path(),
        root.path().join("2026-09-14_KGHM_fundamentals_2/raw.json")
    );
    assert_eq!(read_raw(&first.raw_path()).unwrap().body, document.body);
    assert_eq!(read_raw(&second.raw_path()).unwrap().body, document.body);
    assert!(first.raw_path().with_file_name("error.json").exists());
    assert!(
        !first
            .raw_path()
            .with_file_name("fundamentals.json")
            .exists()
    );
    assert!(first.save_error("overwrite attempt").is_err());
}

#[test]
fn archive_names_identify_the_date_company_and_data_kind() {
    let root = tempfile::tempdir().unwrap();
    let mut document = raw();
    document.source = Source::YahooFinance;
    let history = archive_raw(root.path(), &document).unwrap();
    assert_eq!(
        history.raw_path(),
        root.path().join("2026-09-14_KGHM_history/raw.json")
    );
}

#[test]
fn company_codes_cannot_create_nested_archive_paths() {
    let root = tempfile::tempdir().unwrap();
    let mut document = raw();
    document.instrument.as_mut().unwrap().code = "../KGHM/../escape".into();
    let archive = archive_raw(root.path(), &document).unwrap();
    assert_eq!(
        archive.raw_path().parent().unwrap().parent().unwrap(),
        root.path()
    );
    assert_eq!(read_raw(&archive.raw_path()).unwrap(), document);
}

#[test]
fn simultaneous_collections_keep_all_observations() {
    let root = tempfile::tempdir().unwrap();
    let document = raw();
    let mut paths = std::thread::scope(|scope| {
        let mut workers = Vec::new();
        for _ in 0..8 {
            workers.push(scope.spawn(|| archive_raw(root.path(), &document).unwrap().raw_path()));
        }
        let mut paths = Vec::new();
        for worker in workers {
            paths.push(worker.join().unwrap());
        }
        paths
    });
    paths.sort();
    paths.dedup();
    assert_eq!(paths.len(), 8);
    for path in paths {
        assert_eq!(read_raw(&path).unwrap(), document);
    }
}
