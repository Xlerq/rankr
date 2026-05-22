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
- Analiza historyczna score'u i zestawienie final score z ceną do opisu wyników w pracy.

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
- Zaprojektować `index_membership` dla składu i wag WIG20 z GPW Benchmark.
- Zaprojektować `price_daily` dla danych OHLCV ze Stooq.
- Zaprojektować `fundamental_snapshot` dla danych GPW / Notoria.
- Zaprojektować `macro_observation` dla danych NBP w formacie długim: data, seria, wartość.
- Zaprojektować `score_config` jako konfigurację fundamental-first scoringu.
- Zaprojektować `score_result` jako historię wyników scoringu dla instrumentu i daty.
- Zaprojektować `data_source_log` dla logowania pobrań i importów danych.
- Zaprojektować mapowanie symboli między Stooq, GPW Benchmark, GPW / Notoria i przyszłą bazą danych.
- Przygotować schemat SurrealDB i przykładowy seed.

Gotowe, gdy:

- Model danych obsługuje instrumenty, skład indeksu, ceny, fundamenty, makro, konfiguracje scoringu, wyniki scoringu i logi importu.
- `score_config` odzwierciedla fundamental-first scoring, a nie ranking oparty głównie o price action.
- `score_result` pozwala zapisywać final score dla danej spółki i daty.
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

### Faza 5 - Fundamental-first scoring

- Opisać wzory scoringu w pracy LaTeX.
- Zdefiniować komponenty scoringu oparte głównie o fundamenty:
  - `profitability_score`,
  - `financial_strength_score`,
  - `cashflow_quality_score`,
  - `efficiency_score`,
  - `trend_score` jako lekki filtr techniczny,
  - `macro_context_score` jako opcjonalny kontekst, domyślnie wyłączony w MVP.
- Znormalizować komponenty do skali 0-100.
- Użyć `score_config` z wagami fundamental-first, np. dominujące fundamenty i niski udział trendu.
- Zapisać `final_score`, etykietę i jakość danych w `score_result`.
- Udostępnić endpointy scoringu i rankingu.

Gotowe, gdy:

- Każdy instrument z badanego uniwersum ma score.
- Score jest oparty głównie o dane fundamentalne, a nie o price action.
- Score jest wyjaśnialny przez komponenty.
- Ranking jest sortowany po `final_score`.
- Da się pokazać, z jakiego `score_config` i `fundamental_snapshot` powstał wynik.

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

### Faza 8 - Analiza historyczna score'u

- Zapisywać `score_result` dla wybranych dat historycznych.
- Zestawić historię `final_score` z ceną zamknięcia z `price_daily`.
- Pokazać wykres ceny i historii score'u dla instrumentu.
- Policzyć proste metryki walidacyjne, np. korelację score'u z przyszłą stopą zwrotu albo średni future return dla grup o wysokim i niskim score.
- Opisać ograniczenia: fundamenty są okresowe, publikowane z opóźnieniem i nie muszą działać jako krótkoterminowy sygnał tradingowy.

Gotowe, gdy:

- Da się zobaczyć historię score'u dla instrumentu.
- Da się porównać final score z późniejszym zachowaniem ceny.
- Wyniki nadają się do opisania w pracy bez udawania pełnego systemu tradingowego.
- Ograniczenia analizy historycznej są jasno opisane.

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

- Priorytetem jest działające MVP do fundamentalnego rankingu spółek, nie rozbudowany system tradingowy.
- Konfiguracja wag scoringu pozostaje elementem modelu, ale domyślny scoring jest fundamental-first i globalny; warianty sektorowe mogą być rozszerzeniem.
- Przed commitowaniem większych danych trzeba sprawdzić zasady licencji i redystrybucji źródeł Stooq/GPW.
