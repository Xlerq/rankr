use chrono::{TimeZone, Utc};
use rankr_import::{
    model::{Instrument, RawDocument, Source},
    storage::{archive_raw, read_raw},
};

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

    assert_ne!(first.raw_path(), second.raw_path());
    assert_eq!(read_raw(&first.raw_path()).unwrap().body, document.body);
    assert_eq!(read_raw(&second.raw_path()).unwrap().body, document.body);
    assert!(first.raw_path().with_file_name("error.json").exists());
    assert!(!first.raw_path().with_file_name("fundamentals.json").exists());
    assert!(first.save_error("overwrite attempt").is_err());
}

