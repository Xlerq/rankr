## 6. Faza 0 — formalności i ustawienie projektu

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

### Deliverables

```text
README.md
AI_CONTEXT.md
NAMING_IDEAS.md
docs/project_scope.md
docs/development_plan.md
docs/polish_thesis/temat_pracy.md
docs/polish_thesis/zakres_pracy.md
```

### Definition of Done

- Jest repozytorium.
- Jest robocza nazwa projektu.
- Jest wstępny temat pracy.
- Jest opis zakresu.
- Jest jasne, co jest MVP, a co nie.

---

## 7. Faza 1 — analiza źródeł danych

### Cel

Sprawdzić, skąd i jak pobierać dane EOD dla GPW.

### Zadania

- Spisać listę instrumentów WIG20.
- Sprawdzić symbole instrumentów w Stooq.
- Sprawdzić indeksy: WIG, WIG20, mWIG40, sWIG80.
- Pobrać ręcznie próbki CSV/XLS.
- Sprawdzić format danych.
- Sprawdzić kolumny: date, open, high, low, close, volume.
- Sprawdzić braki danych.
- Opisać ograniczenia źródeł.
- Przygotować sample data w `data/raw/`.

### Deliverables

```text
docs/data_sources.md
data/raw/kgh_sample.csv
data/raw/wig20_sample.csv
docs/polish_thesis/metodyka.md
```

### Definition of Done

- Istnieją próbki danych.
- Wiadomo, jak wygląda format danych.
- Wiadomo, jakie są ograniczenia źródeł.
- Da się ręcznie przejść od pliku danych do planowanego modelu OHLCV.

---

## 8. Faza 2 — projekt danych i bazy

### Cel

Zaprojektować model danych przed implementacją logiki.

### Planowane kolekcje/tabele

```text
instrument
price_daily
score_result
score_config
backtest_result
data_source_log
```

### Zadania

- Zaprojektować `instrument`.
- Zaprojektować `price_daily`.
- Zaprojektować `score_result`.
- Zaprojektować `score_config`.
- Zaprojektować `backtest_result`.
- Zaprojektować `data_source_log`.
- Ustalić unikalność danych cenowych: `instrument_id + date`.
- Ustalić unikalność scoringu: `instrument_id + date + config_id`.
- Przygotować `schema.surql`.
- Przygotować `seed.surql`.

### Deliverables

```text
database/schema.surql
database/seed.surql
docs/architecture.md
```

### Definition of Done

- Wiadomo, gdzie zapisywać instrumenty.
- Wiadomo, gdzie zapisywać ceny.
- Wiadomo, gdzie zapisywać scoringi.
- Wiadomo, gdzie zapisywać konfiguracje.
- Wiadomo, gdzie zapisywać wyniki backtestów.

---

## 9. Faza 3 — minimalny backend

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

### Endpointy startowe

```text
GET /api/health
GET /api/instruments
GET /api/instruments/{symbol}
```

### Deliverables

```text
backend/
shared/
.env.example
```

### Definition of Done

- Backend startuje.
- `/api/health` zwraca JSON.
- Istnieje podstawowa struktura modułów.
- Backend nie zawiera jeszcze niepotrzebnej logiki biznesowej.

---

## 10. Faza 4 — integracja z SurrealDB

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

### Deliverables

```text
backend/src/db/
backend/src/repositories/
database/schema.surql
database/seed.surql
```

### Definition of Done

- Backend łączy się z SurrealDB.
- Można pobrać listę instrumentów.
- Można pobrać pojedynczy instrument.
- Dane pochodzą z bazy, nie z hardcode’u.

---

## 11. Faza 5 — import danych EOD

### Cel

Zbudować pipeline pobierania i zapisywania danych OHLCV.

### Etap 5A — manual CSV import

#### Zadania

- Przyjąć plik CSV.
- Parsować kolumny OHLCV.
- Walidować daty.
- Walidować wartości liczbowe.
- Obsłużyć braki danych.
- Usuwać duplikaty.
- Zapisywać dane do `price_daily`.
- Logować import.

#### Endpointy

```text
POST /api/data/import-csv
GET  /api/prices/{symbol}
```

#### Definition of Done

- Da się zaimportować dane dla jednego instrumentu.
- Ceny są zapisane w SurrealDB.
- API zwraca serię cenową.

### Etap 5B — downloader Stooq/GPW

#### Zadania

- Utworzyć moduł `data_sources/stooq`.
- Utworzyć moduł `data_sources/gpw` jako fallback lub placeholder.
- Dodać mapowanie symboli.
- Pobierać dane dla jednego instrumentu.
- Pobierać dane dla wielu instrumentów.
- Obsłużyć błędy pobierania.
- Logować źródło danych.

#### Endpoint

```text
POST /api/data/update
```

#### Definition of Done

- Da się pobrać dane dla jednego instrumentu.
- Da się pobrać dane dla listy WIG20.
- System nie crashuje przy braku danych.

---

## 12. Faza 6 — moduł analityczny R

### Cel

R liczy wskaźniki i scoring, a Rust potrafi odpalić skrypt przez Rscript.

### Pliki

```text
analytics/io.R
analytics/indicators.R
analytics/scoring.R
analytics/backtest.R
```

### Minimalne wskaźniki

```text
SMA50
SMA200
return_1m
return_3m
return_6m
volatility_20d
max_drawdown
relative_strength_vs_wig20
avg_volume_20d
```

### Komponenty score’u

```text
trend_score
momentum_score
risk_score
volume_score
relative_strength_score
final_score
label
```

### Zadania

- R czyta CSV/JSON.
- R liczy wskaźniki.
- R normalizuje wyniki do skali 0–100.
- R zwraca JSON.
- Rust odpala `Rscript`.
- Rust czyta output JSON.
- Rust zapisuje score do bazy.

### Deliverables

```text
analytics/scoring.R
analytics/indicators.R
backend/src/analytics/rscript_runner.rs
```

### Definition of Done

- Dla jednego instrumentu powstaje final score.
- Wynik jest zapisany w bazie.
- Wynik można pobrać przez API.
- Score jest wyjaśnialny przez komponenty.

---

## 13. Faza 7 — scoring wielu instrumentów i ranking

### Cel

Aplikacja zaczyna wykonywać główną funkcję: ranking instrumentów.

### Zadania

- Pobrać ceny dla wielu instrumentów.
- Wygenerować input dla R.
- Policz scoring dla każdego instrumentu.
- Zapisać `score_result`.
- Dodać endpoint `GET /api/ranking`.
- Dodać sortowanie po `final_score`.
- Dodać filtrowanie po label.
- Dodać endpoint `GET /api/scores/{symbol}`.
- Dodać historię score’u.

### Endpointy

```text
POST /api/score/recalculate
GET  /api/ranking
GET  /api/scores/{symbol}
```

### Definition of Done

- Istnieje ranking WIG20.
- Każdy instrument ma score.
- Można pobrać historię score’u.
- Ranking jest sortowany według final score.

---

## 14. Faza 8 — frontend shell

### Cel

Uruchomić Leptos i podstawowy interfejs.

### Strony

```text
/dashboard
/instrument/{symbol}
/backtest
/settings/scoring
```

### Komponenty

```text
Layout
RankingTable
ScoreBadge
ScoreBreakdown
MetricCard
```

### Zadania

- Utworzyć frontend Leptos.
- Dodać routing.
- Dodać layout.
- Dodać klienta API.
- Dodać dashboard placeholder.
- Dodać tabelę rankingu.
- Obsłużyć loading state.
- Obsłużyć error state.

### Definition of Done

- Frontend startuje.
- Dashboard pobiera `/api/ranking`.
- Ranking jest widoczny w UI.
- UI jest prosty, ale działa.

---

## 15. Faza 9 — Plotters i wykresy

### Cel

Dodać wizualizację danych.

### Kolejność wykresów

1. Close price line chart.
2. SMA50/SMA200 chart.
3. Score history chart.
4. Equity curve.
5. Drawdown chart.
6. Candlestick chart jako opcjonalne rozszerzenie.

### Zadania

- Dodać `PriceChart`.
- Dodać `ScoreHistoryChart`.
- Pobrać dane z API.
- Renderować wykresy przez Plotters.
- Obsłużyć puste dane.
- Dodać podstawową responsywność.

### Definition of Done

- Widok instrumentu pokazuje wykres ceny.
- Widok instrumentu pokazuje historię score’u.
- Wykresy są generowane przez Plotters.
- Nie użyto JS charting libraries.

---

## 16. Faza 10 — widok instrumentu

### Cel

Użytkownik może przeanalizować pojedynczy instrument.

### Widok

```text
/instrument/{symbol}
```

### Elementy widoku

- nazwa instrumentu,
- aktualny final score,
- label,
- breakdown score’u,
- wykres ceny,
- wykres historii score’u,
- podstawowe statystyki,
- ostatnia data aktualizacji.

### Zadania

- Dodać endpoint szczegółów instrumentu.
- Dodać endpoint cen.
- Dodać endpoint score history.
- Dodać frontend page.
- Dodać linki z rankingu do widoku instrumentu.
- Dodać fallback dla braku danych.

### Definition of Done

- Kliknięcie w instrument z rankingu otwiera szczegóły.
- Widok instrumentu jest kompletny.
- Użytkownik rozumie, skąd wziął się score.

---

## 17. Faza 11 — backtesting

### Cel

Zweryfikować scoring historycznie.

### Strategia bazowa

```text
Co miesiąc:
  1. oblicz scoring dla wszystkich instrumentów,
  2. wybierz top N,
  3. trzymaj przez miesiąc,
  4. wykonaj rebalancing,
  5. porównaj z WIG20.
```

### Parametry

```text
start_date
end_date
top_n
rebalance_frequency = monthly
benchmark = WIG20
score_config_id
```

### Metryki

```text
strategy_return
benchmark_return
excess_return
max_drawdown
volatility
sharpe_proxy
win_rate
number_of_rebalances
```

### Zadania

- Dodać `backtest.R`.
- Przygotować input: ceny + historyczne score’y.
- Przygotować output JSON.
- Backend odpala backtest.
- Zapis do `backtest_result`.
- Endpoint `POST /api/backtest`.
- Frontend `/backtest`.
- Wykres equity curve.
- Wykres drawdown.

### Definition of Done

- Da się odpalić backtest.
- Widać wynik strategii.
- Widać benchmark.
- Są metryki.
- Wyniki nadają się do opisania w pracy.

---

## 18. Faza 12 — konfiguracja scoringu

### Cel

Umożliwić zmianę wag modelu.

### Parametry

```text
trend_weight
momentum_weight
risk_weight
volume_weight
relative_strength_weight
```

### Zadania

- Endpoint pobierania konfiguracji.
- Endpoint zapisywania konfiguracji.
- Walidacja sumy wag.
- UI settings.
- Recalculate score with config.
- Backtest with selected config.

### Definition of Done

- Użytkownik może zmienić wagi.
- Ranking można przeliczyć z nowymi wagami.
- Backtest można uruchomić dla wybranej konfiguracji.

---

## 19. Faza 13 — testy

### Cel

Sprawdzić krytyczne części systemu.

### Testy backendu

- healthcheck,
- parsing CSV,
- walidacja danych,
- score label mapping,
- repository functions,
- API response shape.

### Testy R

- scoring dla krótkiej serii danych,
- brakujące wartości,
- max drawdown,
- normalizacja 0–100,
- output JSON.

### Testy UI

- dashboard ładuje ranking,
- instrument detail działa,
- wykres nie crashuje na pustych danych,
- backtest wyświetla wynik.

### Deliverables

```text
tests/
docs/testing.md
docs/polish_thesis/testy.md
```

### Definition of Done

- Istnieją testy dla krytycznych elementów.
- Istnieje opis testów w dokumentacji.
- Istnieją screenshoty wyników.

---

## 20. Faza 14 — polish, demo i zamknięcie aplikacji

### Cel

Zamrozić ficzery i przygotować projekt do pokazania.

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

### Definition of Done

- Projekt da się uruchomić z instrukcji.
- Istnieją dane demo.
- Istnieją screenshoty.
- Istnieje stabilna wersja do obrony.

---

## 21. Faza 15 — pisanie pracy inżynierskiej

### Cel

Napisać pracę równolegle z projektem, a nie po projekcie.

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

W pliku:

```text
docs/polish_thesis/thesis_log.md
```

Zapisywać:

- co zostało zrobione,
- jakie decyzje techniczne podjęto,
- jakie problemy wystąpiły,
- co nadaje się do pracy,
- jakie screenshoty/wykresy powstały.

### Definition of Done

- Jest pełny draft pracy.
- Są screeny aplikacji.
- Są diagramy.
- Są wyniki testów.
- Są wyniki backtestu.
- Jest bibliografia.
- Praca jest sformatowana zgodnie z wymaganiami uczelni/promotora.

---
