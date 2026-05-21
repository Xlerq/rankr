# Rankr — główna roadmapa projektu inżynierskiego

Roadmapa dla projektu **Rankr** — aplikacji webowej do scoringu i rankingu instrumentów finansowych GPW.

Projekt ma służyć jako praca inżynierska oraz portfolio techniczne. Główna zasada: **najpierw działający pipeline danych i MVP, potem rozszerzenia i polish**.

---

## 1. Cel projektu

Celem projektu jest zaprojektowanie i zaimplementowanie aplikacji webowej umożliwiającej:

- import dziennych danych EOD dla instrumentów GPW,
- zapis danych OHLCV w bazie,
- obliczanie scoringu techniczno-ryzykowego,
- ranking instrumentów,
- prezentację wyników w dashboardzie,
- wizualizację ceny i historii score’u,
- prosty backtest strategii rankingowej,
- porównanie wyników z benchmarkiem WIG20.

Projekt **nie jest trading botem**, nie składa zleceń i nie stanowi rekomendacji inwestycyjnej.

---

## 2. Stack technologiczny

| Warstwa | Technologia | Rola |
|---|---|---|
| Frontend | Rust + Leptos + WebAssembly | dashboard, ranking, widok instrumentu |
| Wykresy | Plotters | wykres ceny, historia score’u, equity curve, drawdown |
| Backend | Rust + Axum + Tokio | API, logika aplikacji, orkiestracja |
| Baza danych | SurrealDB | instrumenty, OHLCV, scoringi, konfiguracje, backtesty |
| Analityka | R + Rscript | wskaźniki, scoring, backtest |
| Wymiana danych | JSON / CSV | komunikacja Rust ↔ R |
| Dane | Stooq / GPW EOD | dane dzienne dla GPW/WIG20 |
| Dokumentacja | Markdown + LaTeX | repozytorium i praca inżynierska |

---

## 3. Zakres MVP

MVP musi zawierać:

1. Listę instrumentów WIG20.
2. Import danych EOD.
3. Zapis danych OHLCV w SurrealDB.
4. Moduł R liczący podstawowe wskaźniki.
5. Moduł R liczący scoring 0–100.
6. Backend Axum z endpointami API.
7. Frontend Leptos.
8. Ranking instrumentów.
9. Widok pojedynczego instrumentu.
10. Wykres ceny w Plotters.
11. Wykres historii score’u.
12. Prosty backtest strategii rankingowej.
13. Porównanie strategii z benchmarkiem WIG20.
14. Dokumentację architektury.
15. Opis metodyki scoringu.
16. Opis testów i ograniczeń.

---

## 4. Poza zakresem MVP

Nie implementować w MVP:

- real-time market data,
- intraday data,
- trading bota,
- składania zleceń,
- integracji z brokerem,
- logowania użytkowników,
- machine learning,
- crypto,
- forex,
- opcje,
- pełnych danych fundamentalnych,
- aplikacji mobilnej,
- mikroserwisów,
- Kafki,
- Kubernetesa,
- skomplikowanego systemu autoryzacji.

---

## 5. Główna kolejność pracy

Najważniejsza kolejność:

```text
Dane -> Baza -> Backend -> R scoring -> Ranking -> Frontend -> Wykresy -> Backtest -> Testy -> Praca -> Polish
```

Nie odwracać tej kolejności.

---

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

## 22. Harmonogram 24-tygodniowy

| Tydzień | Kod | Dokumentacja / praca | Efekt |
|---:|---|---|---|
| 1 | repo, katalogi, docs | temat, zakres, cel | projekt ustawiony |
| 2 | próbki danych | źródła danych | wiadomo skąd brać dane |
| 3 | schema SurrealDB | model danych | baza zaprojektowana |
| 4 | Axum healthcheck | opis backendu | backend startuje |
| 5 | SurrealDB integration | opis bazy | zapis/odczyt |
| 6 | CSV import | dane wejściowe | ceny w bazie |
| 7 | Stooq/GPW downloader | jakość danych | automatyczny import |
| 8 | R indicators | scoring methodology | wskaźniki działają |
| 9 | R scoring | normalizacja | score dla instrumentu |
| 10 | Rust ↔ R | integracja | score zapisany w bazie |
| 11 | ranking API | opis API | ranking JSON |
| 12 | Leptos shell | frontend | dashboard shell |
| 13 | ranking table | UI | ranking widoczny |
| 14 | Plotters price chart | wizualizacja | pierwszy wykres |
| 15 | instrument detail | UI opis wyników | widok spółki |
| 16 | score history chart | wizualizacja | historia score’u |
| 17 | backtest.R | metodyka backtestu | symulacja |
| 18 | backtest API/UI | wyniki | equity curve |
| 19 | scoring settings | parametryzacja | zmienne wagi |
| 20 | testy | scenariusze testowe | raport testów |
| 21 | poprawki | rozdział implementacyjny | stabilizacja |
| 22 | freeze kodu | wyniki i screeny | wersja thesis |
| 23 | praca final | formatowanie | pełny draft |
| 24 | obrona/demo | prezentacja | gotowe |

---

## 23. Wersja skrócona 12-tygodniowa

| Tydzień | Priorytet |
|---:|---|
| 1 | temat, repo, dane próbne |
| 2 | baza + backend healthcheck |
| 3 | import CSV + ceny w bazie |
| 4 | R indicators + scoring |
| 5 | ranking API |
| 6 | Leptos dashboard |
| 7 | Plotters price chart + instrument view |
| 8 | score history |
| 9 | backtest |
| 10 | testy + stabilizacja |
| 11 | pisanie pracy |
| 12 | poprawki + demo + final |

---

## 24. Minimalna wersja obronna

Jeżeli projekt trzeba będzie mocno uprościć, minimalna wersja obronna to:

```text
Instrumenty: 5 spółek z WIG20 + WIG20 benchmark
Dane: CSV sample
Backend: Axum API
Baza: SurrealDB
R: scoring + backtest
Frontend: dashboard + instrument detail
Wykresy: cena + score history
Praca: opis projektu, metodyki, implementacji i wyników
```

---

## 25. Docelowa wersja dobra

Wersja, do której warto celować:

```text
Instrumenty: cały WIG20
Benchmarki: WIG20, WIG
Dane: import automatyczny + sample fallback
Scoring: 5 komponentów
Backtest: top 3 / top 5 monthly rebalance
Frontend: ranking, instrument, backtest, settings
Wykresy: price, score history, equity curve, drawdown
Dokumentacja: pełna
```

---

## 26. Największe ryzyka

| Ryzyko | Prawdopodobieństwo | Ból | Kontra |
|---|---:|---:|---|
| Frontend Rust zajmie za dużo czasu | wysokie | duży | prosty UI, zero animacji |
| Plotters w WASM będzie upierdliwy | wysokie | średni/duży | zacząć od line chart |
| Dane GPW/Stooq będą problematyczne | średnie | duży | CSV fallback |
| R integration będzie brzydka | średnie | średni | Rscript + JSON, bez FFI |
| SurrealDB zabierze czas | średnie | średni | prosty model, bez fancy graph |
| Scope creep | bardzo wysokie | katastrofa | MVP freeze |
| Praca zostanie na koniec | wysokie | katastrofa | weekly thesis_log |
| Promotor nie kupi stacku | niskie/średnie | średni | tłumaczyć architekturą, nie hype’em |

---

## 27. Najbliższe 7 dni

1. Ustalić roboczą nazwę repo: `rankr`.
2. Zostawić `README.md`, `AI_CONTEXT.md`, `NAMING_IDEAS.md`.
3. Utworzyć katalogi bez kodu.
4. Napisać `docs/polish_thesis/temat_pracy.md`.
5. Napisać `docs/polish_thesis/zakres_pracy.md`.
6. Napisać `docs/project_scope.md`.
7. Napisać `docs/development_plan.md`.
8. Pobrać ręcznie próbkę danych dla KGH/CDR/WIG20.
9. Zrobić listę symboli WIG20.
10. Przygotować wiadomość do promotora.

---

## 28. Zasada końcowa

Projekt ma wygrać nie ilością technologii, tylko spójnością:

```text
działający pipeline danych
+ wyjaśnialny scoring
+ sensowny ranking
+ prosty backtest
+ czytelna aplikacja
+ dobrze napisana praca
```

Trzymać kolejność:

```text
Dane -> Scoring -> Ranking -> Wizualizacja -> Backtest -> Praca -> Polish
```
