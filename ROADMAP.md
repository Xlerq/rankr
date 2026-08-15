# rankr — roadmap

## Cel

- Zbudować pracę inżynierską w postaci aplikacji webowej do wyjaśnialnego scoringu i rankingu spółek WIG20.
- Łączyć analizę fundamentalną, sektorowy kontekst makroekonomiczny i prosty filtr techniczny.
- Pokazywać wynik, jego składowe, dane wejściowe, źródła oraz datę dostępności informacji.
- Zachować cały kod aplikacji, importu, scoringu i analizy w Rust.
- Traktować wynik jako narzędzie badawcze, a nie rekomendację kupna lub sprzedaży.

## Zakres pracy v1.0

- Pełny skład WIG20 z historią członkostwa potrzebną do analizy.
- Trzy profile scoringowe: spółki niefinansowe, banki i ubezpieczyciele.
- Dane fundamentalne, dzienne OHLCV oraz dane makroekonomiczne dostępne w chwili wyliczenia score'u.
- Ranking dla wybranej daty, scorecard spółki i historia wyniku.
- Wykres ceny, historii score'u i użytego kontekstu makro.
- Małe badanie point-in-time bez deklarowania zdolności do przewidywania rynku.
- Lokalna, odtwarzalna wersja demonstracyjna bez kont, płatności i personalizacji.

## Poza zakresem pracy

- Dane intraday i real-time.
- Wiadomości, analiza sentymentu i modele AI/ML.
- Sygnały `kup`, `sprzedaj`, ceny docelowe i dobór portfela użytkownika.
- Integracja z brokerem i wykonywanie zleceń.
- Alerty, konta, subskrypcje i aplikacja mobilna.
- Spółki spoza WIG20.

## Stan początkowy

- [x] Struktura repozytorium i szkielet pracy w LaTeX.
- [x] Próbki Stooq, GPW Benchmark, GPW/Notoria i NBP.
- [x] Wstępne mapowanie instrumentów WIG20.
- [x] Wstępny schemat i seed SurrealDB zweryfikowane przez rzeczywisty import.
- [x] Backend Axum z `GET /api/health`.
- [ ] Produkcyjny import danych, scoring, API, frontend, testy i analiza historyczna.

## Scoring v1

Domyślna konfiguracja:

```text
final_score = 0.65 * fundamental_score
            + 0.25 * macro_score
            + 0.10 * technical_score
```

- Każda składowa i wynik końcowy mają skalę `0–100`.
- Wagi i wzory są wersjonowane w `score_config`.
- `data_quality_score` jest pokazywany osobno i nie poprawia wyniku spółki.
- Brak danych obniża jakość wyniku zgodnie z jedną udokumentowaną polityką.
- UI pokazuje wartości wskaźników, ich kierunek, wkład do score'u, źródło i świeżość.
- Etykiety wyniku nie używają słów `buy`, `sell` ani ich polskich odpowiedników.

Profile fundamentalne:

- **Spółka niefinansowa:** rentowność, wzrost, zadłużenie, płynność, jakość przepływów, efektywność i wycena.
- **Bank:** rentowność, jakość aktywów, adekwatność kapitałowa, płynność, efektywność kosztowa i wycena.
- **Ubezpieczyciel:** rentowność, wzrost składki i wyniku, wypłacalność, efektywność ubezpieczeniowa i wycena.

Kontekst dodatkowy:

- **Makro:** stopy procentowe, inflacja, aktywność gospodarcza i FX, przypisane do sektorów przez jawną konfigurację ekspozycji.
- **Technika:** położenie względem SMA50/SMA200, długoterminowy trend i relatywna siła wobec WIG20.

## Etapy

### 1. Specyfikacja i metodyka

- [ ] Potwierdzić z promotorem dokładne brzmienie tytułu pracy i poprawić metadane LaTeX.
- [ ] Sformułować problem inżynierski, pytania badawcze i granice interpretacji score'u.
- [ ] Zapisać komplet wzorów, kierunków wskaźników, progów normalizacji i politykę braków danych.
- [ ] Zdefiniować trzy profile fundamentalne i mapowanie sektorów WIG20 do profili.
- [ ] Zdefiniować ekspozycje sektorów na zmienne makro oraz konfigurację `fundamental/macro/technical`.
- [ ] Opisać pochodzenie danych, zasady użycia w pracy i ograniczenia redystrybucji.
- [ ] Umieścić specyfikację metodyki bezpośrednio w `thesis/main.tex`.

### 2. Model domeny i SurrealDB

- [ ] Rozszerzyć workspace o craty dla domeny, importu i scoringu współdzielone z backendem.
- [ ] Rozdzielić raport finansowy od metryk sektorowych, aby obsłużyć trzy profile scoringowe.
- [ ] Dodać `period_end`, `published_at`, `available_at`, wariant raportu, jednostkę, walutę i hash źródła.
- [ ] Dodać historię składu indeksu oraz jawny czas obowiązywania członkostwa.
- [ ] Dodać `score_run` z datą odcięcia, uniwersum, wersją konfiguracji i wersją kodu.
- [ ] Zapisywać komponenty score'u, wkład wskaźników i użyte obserwacje danych.
- [ ] Wprowadzić wersjonowane migracje zamiast jednego ręcznie importowanego schematu.
- [ ] Pokryć ograniczenia OHLCV, unikalność danych i sumę wag testami integracyjnymi.

### 3. Import danych wyłącznie w Rust

- [ ] Utworzyć jedno CLI `rankr_ingest` z poleceniami dla instrumentów, cen, fundamentów i makro.
- [ ] Przenieść parser portfela WIG20 z GPW Benchmark z Python do Rust.
- [ ] Przenieść pobieranie i walidację danych Stooq z Bash do Rust.
- [ ] Uogólnić parser GPW/Notoria na wszystkie spółki WIG20 i przenieść go z Python do Rust.
- [ ] Przenieść import NBP do Rust oraz uzupełnić minimalny zestaw danych makro z oficjalnego źródła.
- [ ] Walidować HTTP, format, typy, jednostki, kompletność, duplikaty i chronologię danych.
- [ ] Zapisywać surowy artefakt, hash, źródło, czas pobrania i wynik importu.
- [ ] Dodać timeouty, ograniczone ponowienia i czytelne błędy bez częściowego nadpisywania danych.
- [ ] Oprzeć testy parserów na lokalnych fixture'ach dla każdego profilu scoringowego.
- [ ] Usunąć zależności i skrypty Python/Bash zastąpione przez przetestowane polecenia Rust.

### 4. Silnik scoringowy

- [ ] Zaimplementować czyste funkcje Rust dla wskaźników, normalizacji i agregacji.
- [ ] Zaimplementować osobne formuły dla spółek niefinansowych, banków i ubezpieczycieli.
- [ ] Zaimplementować sektorowy `macro_score` i wspólny `technical_score`.
- [ ] Obliczać `data_quality_score` oraz listę brakujących lub przestarzałych danych.
- [ ] Zapewnić identyczny wynik dla tych samych danych, konfiguracji i daty odcięcia.
- [ ] Zapisywać score wraz z pełnym śladem pochodzenia i wkładem każdego czynnika.
- [ ] Przetestować skrajne wartości, braki danych, różne jednostki, sumę wag i granice `0–100`.

### 5. Backend API

- [ ] Dodać konfigurację i połączenie Axum z SurrealDB.
- [ ] Rozdzielić liveness `GET /api/health` od readiness `GET /api/ready`.
- [ ] Dodać jednolity model błędów, stan aplikacji, tracing i bezpieczne zamknięcie serwera.
- [ ] Dodać `GET /api/rankings?as_of=`.
- [ ] Dodać `GET /api/instruments/{symbol}`.
- [ ] Dodać `GET /api/instruments/{symbol}/prices`.
- [ ] Dodać `GET /api/instruments/{symbol}/scores`.
- [ ] Dodać `GET /api/macro?as_of=`.
- [ ] Ustabilizować DTO współdzielone z frontendem i przetestować kontrakty API.

### 6. Frontend Leptos

- [ ] Zbudować dashboard z rankingiem WIG20 dla najnowszej lub wybranej daty.
- [ ] Pokazać `final`, `fundamental`, `macro`, `technical` i `data_quality_score`.
- [ ] Dodać sortowanie, podstawowe filtrowanie sektorów i oznaczenie profilu scoringowego.
- [ ] Zbudować widok spółki z rozbiciem score'u, metrykami, źródłami i datami dostępności.
- [ ] Dodać wykres ceny, historii score'u i kontekstu makro przy użyciu Plotters.
- [ ] Obsłużyć loading, brak danych, dane nieaktualne i błędy API.
- [ ] Przygotować prosty, responsywny wygląd odpowiedni do demonstracji pracy.

### 7. Analiza point-in-time

- [ ] Generować historyczne score'y wyłącznie z danych dostępnych w danej chwili.
- [ ] Uwzględniać historyczny skład WIG20 i daty publikacji raportów.
- [ ] Policzyć przyszłe stopy zwrotu dla ustalonych horyzontów bez tworzenia strategii transakcyjnej.
- [ ] Porównać ranking z prostym wariantem fundamental-only oraz zachowaniem indeksu.
- [ ] Policzyć korelację rang Spearmana i różnice między grupami wysokiego i niskiego score'u.
- [ ] Wykonać analizę wrażliwości wag i wpływu brakujących danych.
- [ ] Generować tabele CSV/JSON oraz wykresy Plotters z odtwarzalnego polecenia Rust.
- [ ] Opisać małą próbę, opóźnienia publikacji, zmiany składu i brak podstaw do prognozowania.

### 8. Testy i odtwarzalność

- [ ] Dodać testy jednostkowe domeny i scoringu oraz testy fixture'ów importerów.
- [ ] Dodać testy integracyjne SurrealDB i endpointów Axum.
- [ ] Dodać smoke test głównego przepływu `fixture -> baza -> score -> API`.
- [ ] Dodać CI dla `fmt`, Clippy, testów Rust i kompilacji LaTeX.
- [ ] Przypiąć toolchain Rust i wersję SurrealDB używaną przez projekt.
- [ ] Przygotować jedno polecenie uruchamiające bazę, import, scoring, backend i frontend demo.
- [ ] Uaktualnić README oraz usunąć nieaktualne deklaracje z `AI_CONTEXT.md`.

### 9. Praca i demonstracja

- [ ] Uzupełniać rozdziały o źródłach, architekturze, modelu danych i metodyce wraz z implementacją.
- [ ] Opisać implementację importerów, scoringu, API, frontendu i wykresów.
- [ ] Opisać testy, analizę point-in-time, wyniki i ograniczenia.
- [ ] Dodać literaturę naukową dotyczącą scoringu fundamentalnego i walidacji danych finansowych.
- [ ] Dodać diagram architektury, model danych, przykładowe scorecardy i zrzuty ekranu.
- [ ] Sprawdzić spójność terminologii, źródeł, wzorów, wyników i deklaracji GenAI.
- [ ] Przygotować odtwarzalne demo i finalny PDF pracy.

## Rezultat v1.0

- Każda spółka WIG20 otrzymuje wynik z właściwego profilu sektorowego.
- Każdy wynik można odtworzyć z wersji konfiguracji, kodu i danych dostępnych w danej chwili.
- Ranking i widok spółki wyjaśniają wszystkie składowe bez ukrytej logiki.
- Import, scoring, backend, frontend i analiza działają bez Python i R.
- Najważniejsze parsery, wzory, zapytania bazy i endpointy mają testy.
- Projekt uruchamia się lokalnie z jednej instrukcji i nadaje się do demonstracji.
- Praca opisuje metodę, implementację, wyniki oraz ograniczenia bez twierdzeń o gwarantowanej skuteczności.

## Po pracy — prywatny produkt subskrypcyjny

1. Wydzielić prywatny produkt korzystający z publicznego rdzenia pracy.
2. Przeprowadzić rozmowy najpierw z inwestorami indywidualnymi GPW, później z użytkownikami profesjonalnymi.
3. Zastąpić źródła badawcze danymi z prawem do komercyjnego przetwarzania i prezentacji.
4. Zautomatyzować cykliczny import, przeliczanie score'u, monitoring jakości i kopie zapasowe.
5. Dodać konta, prywatne listy obserwacyjne i alerty o zmianie wyniku oraz jego przyczynach.
6. Dodać płatne plany, limity, rozliczenia i obsługę rezygnacji z subskrypcji.
7. Rozszerzać uniwersum poza WIG20 i dodawać bardziej szczegółowe modele sektorowe.
8. Rozważyć personalizowane rekomendacje dopiero po ustaleniu wymagań prawnych i modelu odpowiedzialności.
