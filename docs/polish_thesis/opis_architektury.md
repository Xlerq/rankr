# Opis architektury

Planowana architektura systemu rankr:
- frontend w Rust, Leptos i WebAssembly,
- backend w Rust z wykorzystaniem Axum i Tokio,
- baza danych SurrealDB,
- wykresy przygotowywane z użyciem Plotters,
- analizy pomocnicze wykonywane w R przez Rscript,
- wymiana danych przez JSON i CSV.

MVP koncentruje się na rankingu GPW/WIG20 dla danych dziennych. Architektura będzie rozwijana stopniowo wraz z dodawaniem kodu źródłowego.
