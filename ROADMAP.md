# rankr — roadmap

## Cel

Lokalna aplikacja webowa do rankingu spółek GPW według względnego potencjału wzrostu w horyzoncie 3–12 miesięcy. Wyższy wynik końcowy oznacza większą szansę na lepszy zwrot niż inne spółki z tego samego koszyka. Wynik nie jest oceną tego, jak dobra jest spółka, ani prognozą zwrotu w procentach. Aplikacja nie stanowi porady inwestycyjnej.

Koszyk MVP obejmuje pełne WIG20. Później możliwe będzie rozszerzenie na inne akcje GPW; 20 spółek nie jest stałym, ostatecznym wszechświatem. Projektowanie tego rozszerzenia pozostaje poza obecnym zakresem.

Widokiem głównym jest tabela spółek. Docelowo dla każdej spółki liczone będą wyniki trzech rodzin, każda z wagą 1. Wynik końcowy będzie sumą ich punktów, będącą miarą potencjału wzrostu i podstawą sortowania. Wagi 1/1/1 są konwencją, nie wynikiem kalibracji. Suma nie podlega normalizacji końcowej ani mapowaniu na oczekiwany procent zwrotu.

Trzy rodziny wyników v1: `fundamental` — kondycja finansowa, wycena i dynamika wyników (wzrost przychodów i zysku, nie sam poziom ROE); `technical` — sezonowość miesiąca kalendarzowego i trend SMA200 z dziennego `close`; `sentiment` — planowany sygnał z ujawnionych pozycji krótkich netto KNF dla emitenta, bez importera i punktacji. v1 używa wag 1/1/1 dla całego WIG20, w tym banków. Wzór `technical` v1 jest zaimplementowany: `technical_score = seasonality_points + trend_points`, każdy składnik to -1, 0 albo +1, bez wag wewnętrznych i osobnego sygnału momentum. Brak sezonowości oznacza `null`, także dla pełnego wyniku technicznego; sezonowość nie dodaje czwartej wagi rodziny. Szablony GICS i progi fundamentów pozostają bez zmian, także węższy zakres punktów Financials. Wspólny ranking czeka na wdrożenie sentymentu.

Frontend, backend i silnik scoringu w pełni w Rust. Stos: Rust, Leptos/WASM, Axum/Tokio, SurrealDB, Plotters. Import, automatyzacja i analiza również w Rust.

Poza zakresem MVP: bot inwestycyjny, integracja z brokerem, dane realtime i uczenie maszynowe.

Dane zbieramy od uruchomienia importera do lutego 2027. Dla cen pobieramy także całą dostępną historię dzienną OHLCV z Yahoo Finance, potrzebną do trendu SMA200 i sezonowości; zakaz pobierania archiwów nie dotyczy cen. Ostatnie kompletne dane dzienne spółek pochodzą z Yahoo Finance, a FX/złoto z TradingView. Fundamenty zbieramy wyłącznie jako bieżące dane GPW/Notoria: najnowsze dostępne raporty, potem kolejne aktualizacje, bez odtwarzania archiwum sprawozdań. Zapisujemy również bieżący skład WIG20.

Po każdym etapie uzupełniamy odpowiadającą mu część `thesis/main.tex` i testujemy wykonany moduł.

## Etap 0. Źródła danych

- [ ] Wybrać źródła bieżących cen, składu WIG20 i fundamentów dla banków, ubezpieczycieli oraz spółek niefinansowych.
- [ ] Opcjonalnie po MVP: wybrać serie makro oraz rynki COT powiązane ze spółkami WIG20.
- [ ] Sprawdzić dostępne pola, daty publikacji, częstotliwość aktualizacji i warunki wykorzystania danych.
- [x] Wybrać publiczne źródło dla spółki: [Rejestr krótkiej sprzedaży KNF](https://rss.knf.gov.pl/rss_pub/). Sprawdzenie 21.10.2026: endpoint `https://rss.knf.gov.pl/rss_pub/JSON` zwraca JSON przez POST formularza z polem `request`, bez klucza API i konta. Obsługuje `limit`/`offset` i filtr ISIN; próba dla BUDIMEX zwróciła dwie pozycje. Pełna odpowiedź zawierała 20 pozycji i obejmowała 6 z 20 ISIN-ów mapy projektu. To interfejs strony, bez gwarancji niezmienności kontraktu. Szczegóły pól i żądania są w `AI_CONTEXT.md`.

Wybrana liczba to suma bieżących, ujawnionych krótkich pozycji netto różnych posiadaczy dla jednego ISIN-u, w procentach wyemitowanego kapitału. Źródło podaje posiadacza, emitenta, ISIN, procent pozycji, datę obliczenia i ostatniej aktualizacji. Nie sumujemy kolejnych historycznych zgłoszeń tej samej pozycji. Według [opisu KNF](https://www.knf.gov.pl/podmioty/Rejestry_i_Ewidencje/rks?articleId=56674&p_id=18) próg publiczny wynosi 0,5% na posiadacza, z kolejnymi progami co 0,1 punktu procentowego; próg zgłoszenia do nadzorcy 0,1% nie oznacza publicznego dostępu do tych mniejszych pozycji. Rejestr nie mierzy opinii wszystkich inwestorów, pozycji długich ani całości krótkiej sprzedaży. Brak wpisu nie oznacza optymizmu tłumu, braku krótkich pozycji ani ich zamknięcia; dla sygnału pozostaje `null`.

Propozycja punktów do uzgodnienia, bez implementacji: porównać dwa kompletne snapshoty w kolejności pobrania dla tego samego emitenta i tego samego niepustego zbioru posiadaczy, z każdą pozycją nadal co najmniej 0,5%. Wzrost sumy ujawnionych procentów → −1, spadek → +1, brak zmiany → 0. Brak porównania, zmiana zbioru posiadaczy, zniknięcie poniżej progu albo niepełna odpowiedź → `null`. Punkt dodatni oznacza tylko spadek ujawnionego zaangażowania krótkiego, nie potwierdzony optymizm. Reguła nie jest skalibrowana. Odrzucamy głosy dotyczące całego WIG20, ogólne pozycjonowanie detaliczne bez identyfikatora konkretnej spółki GPW oraz analizę newsów modelem. Na tym etapie nie powstaje importer sentymentu ani wspólny ranking.

## Etap 1. Podstawa projektu i baza danych

- [ ] Utworzyć workspace Rust i skonfigurować połączenie z SurrealDB.
- [ ] Przygotować schemat i migracje dla instrumentów, składu indeksu, cen, fundamentów oraz wyników scoringu potencjału wzrostu; makro i COT są opcjonalnym rozszerzeniem po MVP.
- [ ] Zapisywać źródło, datę publikacji i pobrania danych; zachowywać kolejne wersje obserwacji.

## Etap 2. Automatyczny import i rozpoczęcie zbierania danych

- [x] Przygotować importer i parser podstawowych fundamentów GPW/Notoria w Rust, z bieżącą listą WIG20 i zapisem surowych oraz odczytanych danych do JSON (`importer/`).
- [x] Dodać `rankr-import history [CODE]`: pełna dostępna historia dzienna spółek WIG20 z Yahoo Finance bez klucza API; `prices [CODE]` pobiera ostatnią kompletną, zakończoną świecę. Mapowanie kodu/ISIN, osobny `adjusted_close`, archiwizacja surowej odpowiedzi, raport pominiętych wadliwych świec i testy lokalne. Indeks WIG20 wymaga osobnego źródła historii i nie jest automatycznie pobierany.
- [ ] Zaimplementować import wszystkich źródeł wybranych dla MVP w Rust i zastąpić dotychczasowe skrypty Python/Bash.
- [ ] Uruchomić harmonogram: ceny po sesji, fundamenty po publikacji; opcjonalnie po MVP makro po publikacji i COT co tydzień z obsługą opóźnień.
- [ ] Dodać walidację, ochronę przed duplikatami, ponowienia i rejestrowanie błędów oraz przerw w zbieraniu danych.
- [ ] Pozostawić importer działający podczas budowy pozostałych modułów; regularnie wykonywać kopię zebranych danych.

## Etap 3. Scoring potencjału wzrostu

- [x] Zaimplementować punkty podstawowych wskaźników `fundamental`: `net_margin`, `operating_margin`, `equity_ratio`, `debt_to_equity`, `ocf_to_net_income`, ze stałymi progami jako konwencją pracy. GICS wybiera szablon i progi, bez rankingu wewnątrz grupy; cały Financials używa tylko `equity_ratio`. `fundamental-points` sumuje dostępne punkty −1/0/+1, a brak wszystkich oznacza `null`; `fundamental-table` pozostaje tabelą surowych wskaźników.
- [ ] Dodać wycenę do rodziny `fundamental` po uzupełnieniu potrzebnych danych.
- [ ] Dodać dynamikę rok do roku do rodziny `fundamental`: wzrost przychodów i zysku na podstawie porównywalnych raportów, nie sam poziom ROE.
- [x] Zaimplementować `technical` v1: `technical_score = seasonality_points + trend_points`, bez wag wewnętrznych i osobnego sygnału momentum. Sezonowość używa wszystkich dostępnych zakończonych analogicznych miesięcy z lat ubiegłych (wystarczy jeden), z progami ±2%; trend to SMA200 z pasem 3%, a mniej niż 200 sesji daje trend 0. Liczyć z `close`, wyłącznie dla `date < as_of`; brak sezonowości i pełnego wyniku oznaczać jako `null`. Komendy offline: `technical` dla jednego pliku i `technical-table` dla rankingu spółek z najnowszych `history.json`.
- [ ] Uzgodnić i zaimplementować `sentiment` z rejestru KNF według ograniczeń opisanych w Etapie 0; wybór źródła nie oznacza jeszcze gotowej punktacji.
- [ ] Po wdrożeniu sentymentu obliczać `final_score = fundamental_score + technical_score + sentiment_score`, z wagami 1/1/1, bez normalizacji końcowej i mapowania na procent zwrotu. Stosować te same wagi dla całego WIG20, w tym banków, oraz prezentować wyniki rodzin i ich wagi. Nie zastępować braku sentymentu zerem.

## Etap 4. Makro sektorowe — opcjonalnie po MVP

Etap nie jest wymagany do ukończenia MVP ani przejścia do etapów 6–9.

- [ ] Zdefiniować wpływ stóp procentowych, inflacji, aktywności gospodarczej i wybranych kursów walut na spółki lub sektory.
- [ ] Zaimplementować reguły dodające lub odejmujące punkty zależnie od otoczenia makro.
- [ ] Włączyć komponent makro do scoringu i zapisywać przyczyny jego punktacji.

## Etap 5. Wpływ COT na scoring — opcjonalnie po MVP

Etap nie jest wymagany do ukończenia MVP ani przejścia do etapów 6–9.

- [ ] Obliczać pozycję netto wybranych kategorii uczestników i jej zmianę między zebranymi raportami.
- [ ] Przypisać rynki COT do odpowiednich spółek i ustalić reguły punktacji.
- [ ] Włączyć komponent COT do scoringu i zapisywać jego wkład oraz datę użytego raportu.

## Etap 6. Backend i zapisywanie rankingów

- [ ] Przeliczać i zapisywać ranking względnego potencjału wzrostu po aktualizacji danych jako sumę punktów trzech rodzin, razem z wersją konfiguracji, składowymi wyniku i ich wagami 1/1/1; stosować ten sam zestaw dla całego WIG20, w tym banków.
- [ ] Zapisywać wyniki modelu v1 (`fundamental`, `technical`, `sentiment`) oraz prostego momentum jako punktu odniesienia przed okresem pomiaru późniejszych zwrotów. Warianty z makro i COT pozostają opcjonalne po MVP.
- [ ] Udostępnić przez Axum ranking, szczegóły spółki, zebrane serie danych i status aktualizacji.

## Etap 7. Frontend

- [ ] Zbudować w Leptos główny widok tabeli pełnego WIG20, domyślnie sortowanej według sumy punktów trzech rodzin z wagami 1/1/1, z wyborem zapisanej daty i filtrowaniem sektorów.
- [ ] Dodać kartę spółki ze składowymi wyniku, wskaźnikami, źródłami i datami danych.
- [ ] Dodać wykresy Plotters dla cen i scoringu oraz obsługę braków danych i błędów aktualizacji; wykresy makro i COT są opcjonalne po MVP.

## Etap 8. Walidacja na zebranych danych

- [ ] Porównać zapisane rankingi z późniejszymi zmianami cen do lutego 2027. Dane fundamentalne ograniczyć do obserwacji zgromadzonych od uruchomienia importera; dla cen korzystać także z pobranego archiwum EOD, używając do sygnałów wyłącznie historii dostępnej na dzień danego rankingu.
- [ ] Dla ocen z pełnymi 3 miesiącami obserwacji policzyć korelację Spearmana i różnicę zwrotów najwyżej oraz najniżej ocenionych spółek; porównać ustalone modele.
- [ ] Wygenerować tabele i wykresy w Rust oraz opisać pokrycie danych, długość obserwacji i wyniki. Ocenę po 6 i 12 miesiącach pozostawić na dalsze zbieranie danych.

## Etap 9. Ukończenie pracy

- [ ] Sprawdzić cały przepływ: automatyczny import -> baza -> scoring -> API -> frontend.
- [ ] Przygotować instrukcję lokalnego uruchomienia i zaktualizować README oraz AI_CONTEXT.md.
- [ ] Ukończyć rozdziały pracy, wygenerować finalny PDF i przygotować prezentację aplikacji oraz wyników walidacji.
