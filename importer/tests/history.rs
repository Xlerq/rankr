use rankr::{
    history::{HistoryError, YahooSymbolMap},
    model::Instrument,
};

#[test]
fn resolves_gpw_codes_and_checks_isin_instead_of_guessing_symbols() {
    let symbols = YahooSymbolMap::bundled().unwrap();
    let kghm = symbols.instrument("kGhM").unwrap();
    assert_eq!(symbols.symbol_for(&kghm).unwrap(), "KGH.WA");
    assert_eq!(
        symbols
            .symbol_for(&symbols.instrument("PKOBP").unwrap())
            .unwrap(),
        "PKO.WA"
    );
    assert!(symbols.instrument("kgh").is_err());
    let mut wrong_isin = kghm;
    wrong_isin.isin = "PL0000000000".into();
    assert!(matches!(
        symbols.symbol_for(&wrong_isin),
        Err(HistoryError::MissingMapping { .. })
    ));
    assert!(
        symbols
            .symbol_for(&Instrument {
                code: "NEWCOMPANY".into(),
                isin: "PL0000000000".into(),
                name: "New company".into(),
            })
            .is_err()
    );
}

#[test]
fn rejects_ambiguous_or_empty_symbol_maps() {
    let header = "gpw_code,isin,name,yahoo_symbol\n";
    let row = "KGHM,PLKGHM000017,KGHM,KGH.WA\n";
    assert!(YahooSymbolMap::from_csv(header).is_err());
    assert!(YahooSymbolMap::from_csv(&format!("{header}{row}{row}")).is_err());
    assert!(YahooSymbolMap::from_csv(&format!("{header}KGHM,PLKGHM000017,KGHM,\n")).is_err());
}
