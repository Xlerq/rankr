# Roadmap

Roadmapa dla `rankr` - projektu inżynierskiego aplikacji webowej do scoringu i rankingowania instrumentów GPW.

MVP:

- GPW/WIG20 end-of-day data.
- Deterministyczny scoring i ranking.
- Prosty backend, frontend, baza danych i wykresy.
- Backtesting wystarczający do opisania wyników w pracy.

Poza MVP:

- Dane real-time.
- Logika bota inwestycyjnego.
- Integracja z brokerem.
- Uczenie maszynowe.

Zasada ogólna: kod źródłowy, testy i dokumentacja powinny powstawać stopniowo, razem z kolejnymi fazami.

---

## Faza 0 - Formalności i ustawienie projektu

### Cel

Ustawić temat, zakres, repozytorium i dokumentację startową.

### Zadania

- Ustalić roboczy temat pracy.
- Przygotować opis projektu dla promotora.
- Potwierdzić zakres MVP.
- Potwierdzić język pracy.
- Potwierdzić akceptację stacku.
- Założyć repozytorium.
- Dodać `README.md`.
- Dodać `AI_CONTEXT.md`.
- Dodać `NAMING_IDEAS.md`.
- Utworzyć strukturę katalogów.
- Utworzyć katalog `docs/polish_thesis/`.

### Rezultaty

- `README.md`
- `AI_CONTEXT.md`
- `NAMING_IDEAS.md`
- `docs/project_scope.md`
- `docs/development_plan.md`
- `docs/polish_thesis/temat_pracy.md`
- `docs/polish_thesis/zakres_pracy.md`

### Gotowe, gdy

- Jest repozytorium.
- Jest robocza nazwa projektu.
- Jest wstępny temat pracy.
- Jest opis zakresu.
- Jest jasne, co jest MVP, a co nie.

---

## Faza 1 - Analiza źródeł danych

### Cel

Sprawdzić, skąd i jak pobierać dane EOD dla GPW.

### Zadania

- Spisać listę instrumentów WIG20.
- Sprawdzić symbole instrumentów w Stooq.
- Sprawdzić indeksy: WIG, WIG20, mWIG40, sWIG80.
- Pobrać ręcznie próbki CSV/XLS.
- Sprawdzić format danych.
- Sprawdzić kolumny: `date`, `open`, `high`, `low`, `close`, `volume`.
- Sprawdzić braki danych.
- Sprawdzić zasady użycia i redystrybucji danych.
- Opisać ograniczenia źródeł.
- Przygotować przykładowe dane w `data/raw/`.

### Rezultaty

- `docs/data_sources.md`
- `data/raw/kgh_sample.csv`
- `data/raw/wig20_sample.csv`
- `docs/polish_thesis/metodyka.md`

### Gotowe, gdy

- Istnieją próbki danych.
- Wiadomo, jak wygląda format danych.
- Wiadomo, jakie są ograniczenia źródeł.
- Wiadomo, czy dane można trzymać w repozytorium.
- Da się ręcznie przejść od pliku danych do planowanego modelu OHLCV.

---

## Faza 2 - Projekt danych, bazy i kontraktów

### Cel

Zaprojektować model danych oraz podstawowe kontrakty przed implementacją logiki.

### Planowane kolekcje/tabele

- `instrument`
- `price_daily`
- `score_result`
- `score_config`
- `backtest_result`
- `data_source_log`

### Zadania

- Zaprojektować `instrument`.
- Zaprojektować `price_daily`.
- Zaprojektować `score_result`.
- Zaprojektować `score_config`.
- Zaprojektować `backtest_result`.
- Zaprojektować `data_source_log`.
- Ustalić unikalność danych cenowych: `instrument_id + date`.
- Ustalić unikalność scoringu: `instrument_id + date + config_id`.
- Opisać podstawowy kontrakt API w `docs/api.md`.
- Przygotować `schema.surql`.
- Przygotować `seed.surql`.

### Rezultaty

- `database/schema.surql`
- `database/seed.surql`
- `docs/architecture.md`
- `docs/api.md`

### Gotowe, gdy

- Wiadomo, gdzie zapisywać instrumenty.
- Wiadomo, gdzie zapisywać ceny.
- Wiadomo, gdzie zapisywać scoringi.
- Wiadomo, gdzie zapisywać konfiguracje.
- Wiadomo, gdzie zapisywać wyniki backtestów.
- Wiadomo, jakie dane frontend będzie pobierał z backendu.

---

## Faza 3 - Minimalny backend

### Cel

Uruchomić backend Axum i pierwszy endpoint.

### Zadania

- Utworzyć Rust workspace.
- Utworzyć crate `backend`.
- Utworzyć crate `shared`.
- Dodać Axum.
- Dodać Tokio.
- Dodać Serde.
- Dodać Tracing.
- Dodać konfigurację `.env`.
- Dodać `AppState`.
- Dodać endpoint `/api/health`.
- Dodać minimalny test healthchecka.

### Endpointy startowe

- `GET /api/health`
- `GET /api/instruments`
- `GET /api/instruments/{symbol}`

### Rezultaty

- `backend/`
- `shared/`
- `.env.example`

### Gotowe, gdy

- Backend startuje.
- `/api/health` zwraca JSON.
- Istnieje podstawowa struktura modułów.
- Backend nie zawiera jeszcze niepotrzebnej logiki biznesowej.
- Healthcheck jest testowany.

---

## Faza 4 - Integracja z SurrealDB

### Cel

Backend zapisuje i odczytuje dane z bazy.

### Zadania

- Odpalić SurrealDB lokalnie.
- Połączyć backend z SurrealDB.
- Dodać repository dla instrumentów.
- Dodać repository dla cen.
- Dodać seed instrumentów.
- Dodać endpoint `GET /api/instruments`.
- Dodać endpoint `GET /api/instruments/{symbol}`.
- Obsłużyć błędy połączenia z bazą.
- Dodać testy repository lub testy integracyjne dla krytycznych zapytań.

### Rezultaty

- `backend/src/db/`
- `backend/src/repositories/`
- `database/schema.surql`
- `database/seed.surql`

### Gotowe, gdy

- Backend łączy się z SurrealDB.
- Można pobrać listę instrumentów.
- Można pobrać pojedynczy instrument.
- Dane pochodzą z bazy, nie z hardcode'u.
- Błędy bazy są obsłużone czytelną odpowiedzią API.

---

## Faza 5 - Import danych EOD

### Cel

Zbudować pipeline pobierania i zapisywania danych OHLCV.

### Etap 5A - Manual CSV import

#### Zadania

- Przyjąć plik CSV.
- Parsować kolumny OHLCV.
- Walidować daty.
- Walidować wartości liczbowe.
- Obsłużyć braki danych.
- Usuwać duplikaty.
- Zapisywać dane do `price_daily`.
- Logować import.
- Dodać testy parsera CSV.

#### Endpointy

- `POST /api/data/import-csv`
- `GET /api/prices/{symbol}`

#### Gotowe, gdy

- Da się zaimportować dane dla jednego instrumentu.
- Ceny są zapisane w SurrealDB.
- API zwraca serię cenową.
- Parser ma testy dla poprawnych i błędnych danych.

### Etap 5B - Downloader Stooq/GPW

#### Zadania

- Utworzyć moduł `data_sources/stooq`.
- Utworzyć moduł `data_sources/gpw` jako fallback lub placeholder.
- Dodać mapowanie symboli.
- Pobierać dane dla jednego instrumentu.
- Pobierać dane dla wielu instrumentów.
- Obsłużyć błędy pobierania.
- Logować źródło danych.

#### Endpoint

- `POST /api/data/update`

#### Gotowe, gdy

- Da się pobrać dane dla jednego instrumentu.
- Da się pobrać dane dla listy WIG20.
- System nie crashuje przy braku danych.
- Log importu pokazuje źródło i status pobrania.

---

## Faza 6 - Metodyka scoringu i moduł analityczny R

### Cel

Zdefiniować explainable scoring, a następnie policzyć go w R i uruchomić przez `Rscript` z Rust.

### Minimalne wskaźniki

- `SMA50`
- `SMA200`
- `return_1m`
- `return_3m`
- `return_6m`
- `volatility_20d`
- `max_drawdown`
- `relative_strength_vs_wig20`
- `avg_volume_20d`

### Komponenty score'u

- `trend_score`
- `momentum_score`
- `risk_score`
- `volume_score`
- `relative_strength_score`
- `final_score`
- `label`

### Pliki

- `analytics/io.R`
- `analytics/indicators.R`
- `analytics/scoring.R`
- `analytics/backtest.R`

### Zadania

- Opisać wzory i założenia w `docs/scoring_methodology.md`.
- Ustalić progi etykiet score'u.
- R czyta CSV/JSON.
- R liczy wskaźniki.
- R normalizuje wyniki do skali 0-100.
- R zwraca JSON.
- Rust odpala `Rscript`.
- Rust czyta output JSON.
- Rust zapisuje score do bazy.
- Dodać testy dla wskaźników i normalizacji.

### Rezultaty

- `docs/scoring_methodology.md`
- `analytics/scoring.R`
- `analytics/indicators.R`
- `backend/src/analytics/rscript_runner.rs`

### Gotowe, gdy

- Dla jednego instrumentu powstaje final score.
- Wynik jest zapisany w bazie.
- Wynik można pobrać przez API.
- Score jest wyjaśnialny przez komponenty.
- Wzory scoringu są opisane w dokumentacji.

---

## Faza 7 - Scoring wielu instrumentów i ranking

### Cel

Aplikacja zaczyna wykonywać główną funkcję: ranking instrumentów.

### Zadania

- Pobrać ceny dla wielu instrumentów.
- Wygenerować input dla R.
- Policzyć scoring dla każdego instrumentu.
- Zapisać `score_result`.
- Dodać endpoint `GET /api/ranking`.
- Dodać sortowanie po `final_score`.
- Dodać filtrowanie po `label`.
- Dodać endpoint `GET /api/scores/{symbol}`.
- Dodać historię score'u.
- Dodać testy sortowania i filtrowania rankingu.

### Endpointy

- `POST /api/score/recalculate`
- `GET /api/ranking`
- `GET /api/scores/{symbol}`

### Gotowe, gdy

- Istnieje ranking WIG20.
- Każdy instrument ma score.
- Można pobrać historię score'u.
- Ranking jest sortowany według `final_score`.
- Ranking jest wystarczająco opisany do użycia w pracy.

---

## Faza 8 - Frontend shell

### Cel

Uruchomić Leptos i podstawowy interfejs.

### Strony

- `/dashboard`
- `/instrument/{symbol}`
- `/backtest`
- `/settings/scoring`

### Komponenty

- `Layout`
- `RankingTable`
- `ScoreBadge`
- `ScoreBreakdown`
- `MetricCard`

### Zadania

- Utworzyć frontend Leptos.
- Dodać routing.
- Dodać layout.
- Dodać klienta API.
- Dodać dashboard placeholder.
- Dodać tabelę rankingu.
- Obsłużyć loading state.
- Obsłużyć error state.

### Gotowe, gdy

- Frontend startuje.
- Dashboard pobiera `/api/ranking`.
- Ranking jest widoczny w UI.
- UI jest prosty, ale działa.

---

## Faza 9 - Plotters i wykresy

### Cel

Dodać wizualizację danych.

### Kolejność wykresów

1. Wykres ceny zamknięcia.
2. Wykres SMA50/SMA200.
3. Wykres historii score'u.
4. Wykres equity curve.
5. Wykres drawdown.
6. Wykres candlestick jako opcjonalne rozszerzenie.

### Zadania

- Dodać `PriceChart`.
- Dodać `ScoreHistoryChart`.
- Pobrać dane z API.
- Renderować wykresy przez Plotters.
- Obsłużyć puste dane.
- Dodać podstawową responsywność.

### Gotowe, gdy

- Widok instrumentu pokazuje wykres ceny.
- Widok instrumentu pokazuje historię score'u.
- Wykresy są generowane przez Plotters.
- Nie użyto JS charting libraries.

---

## Faza 10 - Widok instrumentu

### Cel

Użytkownik może przeanalizować pojedynczy instrument.

### Widok

- `/instrument/{symbol}`

### Elementy widoku

- Nazwa instrumentu.
- Aktualny final score.
- Label.
- Breakdown score'u.
- Wykres ceny.
- Wykres historii score'u.
- Podstawowe statystyki.
- Ostatnia data aktualizacji.

### Zadania

- Dodać endpoint szczegółów instrumentu.
- Dodać endpoint cen.
- Dodać endpoint score history.
- Dodać frontend page.
- Dodać linki z rankingu do widoku instrumentu.
- Dodać fallback dla braku danych.

### Gotowe, gdy

- Kliknięcie w instrument z rankingu otwiera szczegóły.
- Widok instrumentu jest kompletny.
- Użytkownik rozumie, skąd wziął się score.

---

## Faza 11 - Backtesting

### Cel

Zweryfikować scoring historycznie.

### Strategia bazowa

Co miesiąc:

1. Oblicz scoring dla wszystkich instrumentów.
2. Wybierz top N.
3. Trzymaj przez miesiąc.
4. Wykonaj rebalancing.
5. Porównaj z WIG20.

### Parametry

- `start_date`
- `end_date`
- `top_n`
- `rebalance_frequency = monthly`
- `benchmark = WIG20`
- `score_config_id`

### Metryki

- `strategy_return`
- `benchmark_return`
- `excess_return`
- `max_drawdown`
- `volatility`
- `sharpe_proxy`
- `win_rate`
- `number_of_rebalances`

### Zadania

- Opisać założenia w `docs/backtesting_methodology.md`.
- Dodać `backtest.R`.
- Przygotować input: ceny + historyczne score'y.
- Przygotować output JSON.
- Backend odpala backtest.
- Zapis do `backtest_result`.
- Endpoint `POST /api/backtest`.
- Frontend `/backtest`.
- Wykres equity curve.
- Wykres drawdown.

### Gotowe, gdy

- Da się odpalić backtest.
- Widać wynik strategii.
- Widać benchmark.
- Są metryki.
- Wyniki nadają się do opisania w pracy.
- Ograniczenia backtestu są opisane w dokumentacji.

---

## Faza 12 - Konfiguracja scoringu

### Cel

Umożliwić zmianę wag modelu.

### Parametry

- `trend_weight`
- `momentum_weight`
- `risk_weight`
- `volume_weight`
- `relative_strength_weight`

### Zadania

- Endpoint pobierania konfiguracji.
- Endpoint zapisywania konfiguracji.
- Walidacja sumy wag.
- UI settings.
- Przeliczanie score'u z wybraną konfiguracją.
- Backtest z wybraną konfiguracją.

### Gotowe, gdy

- Użytkownik może zmienić wagi.
- Ranking można przeliczyć z nowymi wagami.
- Backtest można uruchomić dla wybranej konfiguracji.

---

## Faza 13 - Testy i walidacja

### Cel

Sprawdzić krytyczne części systemu i zebrać materiał do pracy.

### Testy backendu

- Healthcheck.
- Parsing CSV.
- Walidacja danych.
- Mapowanie etykiet score'u.
- Repository functions.
- API response shape.

### Testy R

- Scoring dla krótkiej serii danych.
- Brakujące wartości.
- Max drawdown.
- Normalizacja 0-100.
- Output JSON.

### Testy UI

- Dashboard ładuje ranking.
- Instrument detail działa.
- Wykres nie crashuje na pustych danych.
- Backtest wyświetla wynik.

### Rezultaty

- `tests/`
- `docs/testing.md`
- `docs/polish_thesis/testy.md`

### Gotowe, gdy

- Istnieją testy dla krytycznych elementów.
- Istnieje opis testów w dokumentacji.
- Istnieją screenshoty wyników.
- Wyniki testów da się opisać w pracy.

---

## Faza 14 - Dopracowanie, demo i zamknięcie aplikacji

### Cel

Zamrozić funkcje i przygotować projekt do pokazania.

### Zadania

- Poprawić README.
- Poprawić AI_CONTEXT.
- Dodać instrukcję uruchomienia.
- Dodać `.env.example`.
- Dodać przykładowe dane.
- Dodać screeny do README.
- Oznaczyć wersję `v0.1-thesis`.
- Przygotować demo scenario.
- Przygotować backup lokalny.
- Stworzyć finalny branch/tag.

### Gotowe, gdy

- Projekt da się uruchomić z instrukcji.
- Istnieją dane demo.
- Istnieją screenshoty.
- Istnieje stabilna wersja do obrony.

---

## Faza 15 - Pisanie pracy inżynierskiej

### Cel

Napisać pracę równolegle z projektem, a nie dopiero po projekcie.

### Kolejność pisania

1. Zakres pracy.
2. Architektura systemu.
3. Model danych.
4. Metodyka scoringu.
5. Implementacja.
6. Testy i wyniki.
7. Podsumowanie.
8. Wstęp.
9. Streszczenie.
10. Bibliografia.
11. Spis rysunków/tabel.

### Co dopisywać co tydzień

W pliku `notes/thesis_log.md` zapisywać:

- Co zostało zrobione.
- Jakie decyzje techniczne podjęto.
- Jakie problemy wystąpiły.
- Co nadaje się do pracy.
- Jakie screenshoty/wykresy powstały.

### Gotowe, gdy

- Jest pełny draft pracy.
- Są screeny aplikacji.
- Są diagramy.
- Są wyniki testów.
- Są wyniki backtestu.
- Jest bibliografia.
- Praca jest sformatowana zgodnie z wymaganiami uczelni/promotora.

---

## Uwagi po analizie

- Roadmapa ma sens jako plan pracy inżynierskiej, bo prowadzi od zakresu i danych do implementacji, walidacji, demo oraz pisania pracy.
- Największe ryzyko to zbyt duży zakres. Jeśli czas zacznie się kurczyć, priorytetem powinny zostać fazy 0-9 oraz minimalny backtest z fazy 11.
- Konfiguracja scoringu z fazy 12 jest wartościowa, ale może być potraktowana jako rozszerzenie po działającym rankingu.
- Testy nie powinny czekać wyłącznie do fazy 13. W roadmapie zostały dopisane także do wcześniejszych faz.
- Przed commitowaniem większych próbek danych warto ustalić zasady licencji i redystrybucji danych Stooq/GPW.
