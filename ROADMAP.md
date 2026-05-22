# Roadmap

Roadmapa dla `rankr` - projektu inżynierskiego aplikacji webowej do scoringu i rankingowania instrumentów GPW.

Dokumentacja pracy dyplomowej jest konsolidowana w LaTeX:

- źródło pracy: `thesis/main.tex`
- docelowy plik wynikowy: `thesis/main.pdf`

## MVP

- Dane GPW/WIG20 end-of-day.
- Deterministyczny scoring i ranking.
- Backend Rust + Axum + Tokio.
- Frontend Rust + Leptos + WebAssembly.
- Baza SurrealDB.
- Wykresy przez Plotters.
- Minimalny backtesting do opisu wyników w pracy.

## Poza MVP

- Dane real-time.
- Logika bota inwestycyjnego.
- Integracja z brokerem.
- Uczenie maszynowe.

## Fazy

### Faza 0 - Repozytorium i zakres

- Ustalić temat pracy.
- Potwierdzić zakres MVP.
- Uporządkować strukturę repozytorium.
- Przygotować główny plik LaTeX pracy.

Gotowe, gdy:

- Repozytorium ma prostą strukturę.
- Istnieje `thesis/main.tex`.
- README i roadmapa są spójne z zakresem MVP.

### Faza 1 - Warstwa badawcza danych

- Sprawdzić Stooq jako źródło danych EOD OHLCV.
- Sprawdzić GPW Benchmark jako źródło składu i wag WIG20.
- Sprawdzić GPW / Notoria jako źródło danych fundamentalnych.
- Sprawdzić NBP jako źródło danych FX, makro i złota.
- Pobrać małe próbki danych dla każdego źródła.
- Sprawdzić format OHLCV.
- Zweryfikować symbole WIG20 i mapowanie symboli między źródłami.
- Ustalić zasady trzymania danych w repozytorium.
- Opisać ograniczenia źródeł danych.

Gotowe, gdy:

- Repozytorium zawiera próbki Stooq, NBP, GPW / Notoria oraz GPW Benchmark albo czytelny fallback.
- Istnieją skrypty pobierające próbki i skrypt walidujący.
- Istnieje `data/raw/wig20_symbols.csv`.
- Wiadomo, jak wyglądają formaty danych.
- Znane są ograniczenia źródeł danych.

### Faza 2 - Model danych i baza

- Zaprojektować `instrument`.
- Zaprojektować `price_daily`.
- Zaprojektować `fundamental_snapshot`.
- Zaprojektować `macro_daily`.
- Zaprojektować `score_result`.
- Zaprojektować `score_config`.
- Zaprojektować `backtest_result`.
- Zaprojektować mapowanie symboli między Stooq, GPW Benchmark, GPW / Notoria i przyszłą bazą danych.
- Przygotować schemat SurrealDB.

Gotowe, gdy:

- Model danych obsługuje ceny, scoring i wyniki backtestów.
- Backend ma jasny kontrakt danych dla frontendu.

### Faza 3 - Backend

- Utworzyć Rust workspace.
- Dodać backend Axum.
- Dodać endpoint `GET /api/health`.
- Połączyć backend z SurrealDB.
- Dodać endpointy instrumentów i cen.

Gotowe, gdy:

- Backend startuje lokalnie.
- Dane są pobierane z bazy, nie z hardcode'u.
- Krytyczne endpointy mają testy.

### Faza 4 - Import danych EOD

- Dodać import CSV.
- Dodać walidację danych OHLCV.
- Dodać usuwanie duplikatów.
- Dodać downloader Stooq.
- Dodać downloader GPW Benchmark.
- Dodać downloader GPW / Notoria.
- Dodać downloader NBP.
- Logować źródło i status importu.

Gotowe, gdy:

- Da się zaimportować dane dla jednego instrumentu.
- Da się pobrać dane dla listy WIG20.
- API zwraca serię cenową.

### Faza 5 - Scoring

- Opisać wzory scoringu w pracy LaTeX.
- Policzyć wskaźniki: trend, momentum, ryzyko, wolumen i relative strength.
- Znormalizować komponenty do skali 0-100.
- Zapisać `final_score` i etykietę.
- Udostępnić endpointy scoringu.

Gotowe, gdy:

- Każdy instrument WIG20 ma score.
- Score jest wyjaśnialny przez komponenty.
- Ranking jest sortowany po `final_score`.

### Faza 6 - Frontend

- Uruchomić Leptos.
- Dodać dashboard.
- Dodać tabelę rankingu.
- Dodać widok instrumentu.
- Dodać loading i error state.

Gotowe, gdy:

- UI pokazuje ranking WIG20.
- Można wejść w szczegóły instrumentu.
- Widok jest prosty, ale działający.

### Faza 7 - Wykresy

- Dodać wykres ceny zamknięcia.
- Dodać wykres SMA50/SMA200.
- Dodać wykres historii score'u.
- Obsłużyć puste dane.

Gotowe, gdy:

- Widok instrumentu pokazuje cenę i historię score'u.
- Wykresy są generowane przez Plotters.

### Faza 8 - Backtesting

- Przygotować bazową strategię top N.
- Porównać wynik z WIG20.
- Policzyć return, drawdown, volatility i prostą miarę Sharpe proxy.
- Pokazać equity curve i drawdown.

Gotowe, gdy:

- Backtest działa dla historycznych danych.
- Wyniki nadają się do opisania w pracy.
- Ograniczenia backtestu są jasno opisane.

### Faza 9 - Testy i demo

- Dodać testy backendu.
- Dodać testy parsera CSV.
- Dodać testy scoringu.
- Przygotować dane demo.
- Przygotować screenshoty do pracy.

Gotowe, gdy:

- Projekt da się uruchomić z instrukcji.
- Najważniejsze elementy są przetestowane.
- Jest stabilna wersja do pokazania.

### Faza 10 - Praca inżynierska

- Uzupełniać `thesis/main.tex` równolegle z implementacją.
- Opisać cel, zakres, architekturę, model danych, scoring, testy i wyniki.
- Dodać bibliografię.
- Wygenerować finalny PDF.

Gotowe, gdy:

- Istnieje pełny draft pracy.
- `thesis/main.pdf` jest gotowy do oddania.
- Praca jest zgodna z wymaganiami uczelni i promotora.

## Uwagi

- Priorytetem jest działające MVP, nie rozbudowany system tradingowy.
- Konfiguracja wag scoringu może być rozszerzeniem po działającym rankingu.
- Przed commitowaniem większych danych trzeba sprawdzić zasady licencji i redystrybucji źródeł Stooq/GPW.
