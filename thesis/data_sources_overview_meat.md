# Dane w projekcie `rankr`

Cel: zebrać dane dla spółek GPW/WIG20, zapisać je w SurrealDB i policzyć ranking typu **fundamental-first**.

## Źródła danych

| Źródło | Co pobieram | Pliki w repo | Skrypt | Kolekcja w DB | Użycie |
|---|---|---|---|---|---|
| Stooq | dzienne ceny OHLCV | `data/raw/kgh_stooq_sample.csv`, `data/raw/wig20_stooq_sample.csv` | `scripts/fetch_stooq_samples.sh` | `price_daily` | wykres ceny, SMA50/SMA200, `trend_score`, porównanie score'u z ceną |
| GPW Benchmark | skład i wagi WIG20 | `data/raw/wig20_portfolio_gpwbenchmark_sample.json`, `data/raw/wig20_symbols.csv` | `scripts/fetch_gpwbenchmark_wig20_portfolio.py` | `instrument`, `index_membership` | lista analizowanych spółek, wagi indeksu, mapowanie symboli |
| GPW / Notoria | dane fundamentalne spółek | `data/raw/11bit_gpw_notoria_sample.html`, `data/raw/11bit_gpw_notoria_sample.json` | `scripts/fetch_gpw_notoria_sample.py` | `fundamental_snapshot` | główna podstawa scoringu fundamentalnego |
| NBP | USD/PLN, EUR/PLN, cena złota | `data/raw/nbp_usdpln_sample.json`, `data/raw/nbp_eurpln_sample.json`, `data/raw/nbp_gold_sample.json` | `scripts/fetch_nbp_samples.sh` | `macro_observation` | opcjonalny kontekst makro, `macro_context_score` |

## Stooq

**Dane:**

- data sesji,
- open,
- high,
- low,
- close,
- volume.

**Do czego:**

- historia ceny instrumentu,
- wykres ceny zamknięcia,
- SMA50 / SMA200,
- lekki `trend_score`,
- porównanie `final_score` z późniejszą stopą zwrotu.

**Uwaga:** dane cenowe nie są rdzeniem rankingu. W MVP mają wspierać analizę, nie dominować scoring.

## GPW Benchmark

**Dane:**

- skład indeksu WIG20,
- kod ISIN,
- nazwa instrumentu,
- udział w indeksie,
- udział w obrotach,
- spread,
- data obowiązywania składu,
- mapowanie symboli.

**Do czego:**

- ustalenie badanego koszyka spółek,
- zapis składu WIG20 w `index_membership`,
- przypisanie wag indeksowych,
- połączenie symboli między Stooq, GPW Benchmark, GPW / Notoria i bazą.

**Uwaga:** skład WIG20 zmienia się w czasie, więc trzeba trzymać datę `as_of`.

## GPW / Notoria

**Dane:**

- okres raportowy,
- przychody,
- zysk operacyjny,
- zysk netto,
- EBITDA,
- aktywa,
- kapitał własny,
- zobowiązania,
- cash flow operacyjny,
- cash flow inwestycyjny,
- cash flow finansowy,
- wskaźniki typu ROE, ROA, płynność, zadłużenie — jeśli dostępne lub możliwe do policzenia.

**Do czego:**

- `profitability_score`,
- `financial_strength_score`,
- `cashflow_quality_score`,
- `efficiency_score`,
- `final_score`,
- opis, dlaczego dana spółka dostała konkretny wynik.

**Uwaga:** to jest najważniejsze źródło dla MVP. Ranking ma być fundamentalny, nie oparty głównie o price action.

## NBP

**Dane:**

- `USDPLN`,
- `EURPLN`,
- `GOLD_PLN`,
- data obserwacji,
- wartość,
- jednostka.

**Do czego:**

- opcjonalny `macro_context_score`,
- kontekst dla spółek zależnych od walut, eksportu, importu lub surowców,
- rozszerzenie analizy w pracy.

**Uwaga:** w MVP makro może być wyłączone albo mieć małą wagę.

## Kolekcje w SurrealDB

| Kolekcja | Rola |
|---|---|
| `instrument` | spółka / instrument giełdowy |
| `index_membership` | przynależność instrumentu do WIG20 i jego waga |
| `price_daily` | dzienne dane OHLCV |
| `fundamental_snapshot` | dane fundamentalne z raportu / okresu |
| `macro_observation` | obserwacje makro, np. kursy walut i złoto |
| `score_config` | konfiguracja wag scoringu |
| `score_result` | wynik scoringu dla spółki i daty |
| `data_source_log` | log pobrań i importów danych |

## Scoring

Domyślna logika: **fundamental-first**.

| Komponent | Główne źródło | Rola |
|---|---|---|
| `profitability_score` | GPW / Notoria | rentowność |
| `financial_strength_score` | GPW / Notoria | bilans i zadłużenie |
| `cashflow_quality_score` | GPW / Notoria | jakość przepływów pieniężnych |
| `efficiency_score` | GPW / Notoria | efektywność aktywów i kapitału |
| `trend_score` | Stooq | pomocniczy filtr techniczny |
| `macro_context_score` | NBP | opcjonalny kontekst makro |

Wynik końcowy:

- skala `0-100`,
- zapis w `score_result`,
- ranking sortowany po `final_score`,
- wynik ma być wyjaśnialny przez komponenty.

## Ograniczenia

- Dane są próbkami, nie pełną hurtownią danych.
- Stooq daje dane dzienne, nie real-time.
- Fundamenty są okresowe i publikowane z opóźnieniem.
- Parser GPW / Notoria może wymagać korekt przy zmianie HTML.
- Symbole trzeba mapować między źródłami.
- Przed dodaniem większych zbiorów do repo trzeba sprawdzić licencje i redystrybucję.
- Projekt nie daje rekomendacji inwestycyjnych i nie wykonuje transakcji.
