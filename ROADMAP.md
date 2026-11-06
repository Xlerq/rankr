# rankr — roadmap

## Cel

Lokalna aplikacja webowa do rankingu spółek GPW według względnego potencjału wzrostu w horyzoncie 3–12 miesięcy. Wyższy wynik końcowy oznacza większą szansę na lepszy zwrot niż inne spółki z tego samego koszyka. Wynik nie jest oceną tego, jak dobra jest spółka, ani prognozą zwrotu w procentach. Aplikacja nie stanowi porady inwestycyjnej.

Koszyk MVP obejmuje pełne WIG20. Później możliwe będzie rozszerzenie na inne akcje GPW; 20 spółek nie jest stałym, ostatecznym wszechświatem. Projektowanie tego rozszerzenia pozostaje poza obecnym zakresem.

Jedynym programem jest `rankr`. Po uruchomieniu uzupełnia `data/collected`, zapisuje snapshot rankingu i otwiera `http://127.0.0.1:3000/` z tabelą rankingu na dziś UTC. Nie ma osobnych komend `rankr-import`.

Widokiem głównym jest tabela spółek. Dla każdej spółki liczone są wyniki trzech rodzin, każda z wagą 1: `final_score = fundamental_score + technical_score + sentiment_score`. Brak którejkolwiek rodziny daje `final_score = null`, a nie zero. Wynik końcowy jest miarą potencjału wzrostu i podstawą sortowania. Wagi 1/1/1 są konwencją, nie wynikiem kalibracji. Suma nie podlega normalizacji końcowej ani mapowaniu na oczekiwany procent zwrotu.

Trzy rodziny wyników v1: `fundamental` — zaimplementowana kondycja finansowa według szablonów GICS; wycena i dynamika rok do roku (wzrost przychodów i zysku, nie sam poziom ROE) pozostają do zrobienia; `technical` — sezonowość miesiąca kalendarzowego i trend SMA200 z dziennego `close`; `sentiment` — zaimplementowany sygnał ze zmiany sumy ujawnionych pozycji krótkich netto KNF między dwoma snapshotami emitenta. v1 używa wag 1/1/1 dla całego WIG20, w tym banków. Wzór `technical` v1 jest zaimplementowany: `technical_score = seasonality_points + trend_points`, każdy składnik to -1, 0 albo +1, bez wag wewnętrznych i osobnego sygnału momentum. Brak sezonowości oznacza `null`, także dla pełnego wyniku technicznego; sezonowość nie dodaje czwartej wagi rodziny. Szablony GICS i progi fundamentów pozostają bez zmian, także węższy zakres punktów Financials. Wspólny ranking trzech rodzin jest zaimplementowany.

Docelowo frontend, backend i silnik scoringu w pełni w Rust. Planowany stos: Rust, Leptos/WASM, Axum/Tokio, SurrealDB, Plotters. Import, automatyzacja i analiza również w Rust. Obecna tabela działa; SurrealDB, frontend w Leptos i wykresy pozostają do zrobienia.

Poza zakresem MVP: bot inwestycyjny, integracja z brokerem, dane realtime i uczenie maszynowe.

Dane zbieramy od uruchomienia importera do lutego 2027. Dla cen pobieramy także całą dostępną historię dzienną OHLCV z Yahoo Finance, potrzebną do trendu SMA200 i sezonowości; zakaz pobierania archiwów nie dotyczy cen. Ostatnie kompletne dane dzienne spółek pochodzą z Yahoo Finance, a FX/złoto z TradingView. Fundamenty zbieramy wyłącznie jako bieżące dane GPW/Notoria: najnowsze dostępne raporty, potem kolejne aktualizacje, bez odtwarzania archiwum sprawozdań. Zapisujemy również bieżący skład WIG20.

Po każdym etapie uzupełniamy odpowiadającą mu część `thesis/main.tex` i testujemy wykonany moduł.

## Etap 0. Źródła danych

- [ ] Wybrać źródła bieżących cen, składu WIG20 i fundamentów dla banków, ubezpieczycieli oraz spółek niefinansowych.
- [ ] Opcjonalnie po MVP: wybrać serie makro oraz rynki COT powiązane ze spółkami WIG20.
- [ ] Sprawdzić dostępne pola, daty publikacji, częstotliwość aktualizacji i warunki wykorzystania danych.
- [x] Wybrać publiczne źródło dla spółki: [Rejestr krótkiej sprzedaży KNF](https://rss.knf.gov.pl/rss_pub/). Sprawdzenie 21.10.2026: endpoint `https://rss.knf.gov.pl/rss_pub/JSON` zwraca JSON przez POST formularza z polem `request`, bez klucza API i konta. Obsługuje `limit`/`offset` i filtr ISIN; próba dla BUDIMEX zwróciła dwie pozycje. Pełna odpowiedź zawierała 20 pozycji i obejmowała 6 z 20 ISIN-ów mapy projektu. To interfejs strony, bez gwarancji niezmienności kontraktu. Szczegóły pól i żądania są w `AI_CONTEXT.md`.

Wybrana liczba to suma bieżących, ujawnionych krótkich pozycji netto różnych posiadaczy dla jednego ISIN-u, w procentach wyemitowanego kapitału. Źródło podaje posiadacza, emitenta, ISIN, procent pozycji, datę obliczenia i ostatniej aktualizacji. Nie sumujemy kolejnych historycznych zgłoszeń tej samej pozycji. Według [opisu KNF](https://www.knf.gov.pl/podmioty/Rejestry_i_Ewidencje/rks?articleId=56674&p_id=18) próg publiczny wynosi 0,5% na posiadacza, z kolejnymi progami co 0,1 punktu procentowego; próg zgłoszenia do nadzorcy 0,1% nie oznacza publicznego dostępu do tych mniejszych pozycji. Rejestr nie mierzy opinii wszystkich inwestorów, pozycji długich ani całości krótkiej sprzedaży. Brak wpisu nie oznacza optymizmu tłumu, braku krótkich pozycji ani ich zamknięcia; punkty zależą od porównania snapshotów.

Reguła punktów v1: porównać sumy pozycji dwóch snapshotów o tym samym ISIN, z nowszym `fetched_at`. Wzrost sumy → −1, spadek → +1, brak zmiany → 0, także gdy zmienia się lista posiadaczy. Nie ma filtra 0,5% w punktacji ani minimalnego odstępu między pobraniami. Jedna pusta lista ma sumę 0; dwie puste listy albo brak pary dają `null`. Punkt dodatni oznacza tylko spadek ujawnionego zaangażowania krótkiego, nie potwierdzony optymizm. Reguła jest przyjętą konwencją, bez kalibracji. Odrzucamy głosy dotyczące całego WIG20, ogólne pozycjonowanie detaliczne bez identyfikatora konkretnej spółki GPW oraz analizę newsów modelem.

## Etap 1. Podstawa projektu i baza danych

- [ ] Utworzyć workspace Rust i skonfigurować połączenie z SurrealDB.
- [ ] Przygotować schemat i migracje dla instrumentów, składu indeksu, cen, fundamentów oraz wyników scoringu potencjału wzrostu; makro i COT są opcjonalnym rozszerzeniem po MVP.
- [ ] Zapisywać źródło, datę publikacji i pobrania danych; zachowywać kolejne wersje obserwacji.

## Etap 2. Automatyczny import i rozpoczęcie zbierania danych

- [x] Przygotować importer i parser podstawowych fundamentów GPW/Notoria w Rust, z bieżącą listą WIG20 i zapisem surowych oraz odczytanych danych do JSON (`importer/`).
- [x] Pobierać w `rankr` pełną dostępną historię dzienną spółek WIG20 z Yahoo Finance bez klucza API i przy starcie dociągać brakujące świece do dzisiaj. Mapowanie kodu/ISIN, osobny `adjusted_close`, archiwizacja surowej odpowiedzi, raport pominiętych wadliwych świec i testy lokalne. Indeks WIG20 wymaga osobnego źródła historii i nie jest automatycznie pobierany.
- [x] Zintegrować w jednym programie `rankr` zbieranie fundamentów GPW/Notoria, historii Yahoo oraz snapshotów shortów KNF; uzupełniać `data/collected` przy uruchomieniu, bez osobnych komend importu.
- [ ] Uruchomić harmonogram: ceny po sesji, fundamenty po publikacji; opcjonalnie po MVP makro po publikacji i COT co tydzień z obsługą opóźnień.
- [ ] Dodać walidację, ochronę przed duplikatami, ponowienia i rejestrowanie błędów oraz przerw w zbieraniu danych.
- [ ] Pozostawić importer działający podczas budowy pozostałych modułów; regularnie wykonywać kopię zebranych danych.

## Etap 3. Scoring potencjału wzrostu

- [x] Zaimplementować punkty podstawowych wskaźników `fundamental`: `net_margin`, `operating_margin`, `equity_ratio`, `debt_to_equity`, `ocf_to_net_income`, ze stałymi progami jako konwencją pracy. GICS wybiera szablon i progi, bez rankingu wewnątrz grupy; cały Financials używa tylko `equity_ratio`. Wynik fundamentalny sumuje dostępne punkty −1/0/+1, a brak wszystkich oznacza `null`; zachowywane są również surowe wskaźniki.
- [ ] Dodać wycenę do rodziny `fundamental` po uzupełnieniu potrzebnych danych.
- [ ] Dodać dynamikę rok do roku do rodziny `fundamental`: wzrost przychodów i zysku na podstawie porównywalnych raportów, nie sam poziom ROE.
- [x] Zaimplementować `technical` v1: `technical_score = seasonality_points + trend_points`, bez wag wewnętrznych i osobnego sygnału momentum. Sezonowość używa wszystkich dostępnych zakończonych analogicznych miesięcy z lat ubiegłych (wystarczy jeden), z progami ±2%; trend to SMA200 z pasem 3%, a mniej niż 200 sesji daje trend 0. Liczyć z `close`, wyłącznie dla `date < as_of`; brak sezonowości i pełnego wyniku oznaczać jako `null`. Scoring działa w programie `rankr` na zapisanej historii cen.
- [x] Zaimplementować `sentiment_score` ze zmiany sumy ujawnionych shortów KNF między dwoma snapshotami według reguł i ograniczeń opisanych w Etapie 0.
- [x] Obliczać wspólny ranking: `final_score = fundamental_score + technical_score + sentiment_score`, z wagami 1/1/1, bez normalizacji końcowej i mapowania na procent zwrotu. Stosować te same wagi dla całego WIG20, w tym banków. Brak którejkolwiek rodziny oznacza `final_score = null`, a nie zero.

## Etap 4. Makro sektorowe — opcjonalnie po MVP

Etap nie jest wymagany do ukończenia MVP ani przejścia do etapów 6–9.

- [ ] Zdefiniować wpływ stóp procentowych, inflacji, aktywności gospodarczej i wybranych kursów walut na spółki lub sektory.
- [ ] Zaimplementować reguły dodające lub odejmujące punkty zależnie od otoczenia makro.
- [ ] Włączyć komponent makro do scoringu i zapisywać przyczyny jego punktacji.

## Etap 5. Wpływ COT na scoring — opcjonalnie po MVP

Etap nie jest wymagany do ukończenia MVP ani przejścia do etapów 6–9.

Strona shortów KNF dla spółki nie zastępuje COT; widok rejestru KNF należy do etapu 7, a nie do tego etapu.

- [ ] Obliczać pozycję netto wybranych kategorii uczestników i jej zmianę między zebranymi raportami.
- [ ] Przypisać rynki COT do odpowiednich spółek i ustalić reguły punktacji.
- [ ] Włączyć komponent COT do scoringu i zapisywać jego wkład oraz datę użytego raportu.

## Etap 6. Backend i zapisywanie rankingów

- [x] Przy starcie `rankr`, po uzupełnieniu danych, przeliczać i zapisywać snapshot rankingu na dziś UTC jako sumę punktów trzech rodzin z wagami 1/1/1; stosować ten sam zestaw dla całego WIG20, w tym banków. Brak rodziny daje `final_score = null`.
- [x] Po uruchomieniu otwierać `http://127.0.0.1:3000/` z tabelą dzisiejszego rankingu UTC.
- [ ] Uzupełnić zapis rankingu o wersję konfiguracji, składowe wyniku i ich wagi 1/1/1.
- [ ] Zapisywać wyniki modelu v1 (`fundamental`, `technical`, `sentiment`) oraz prostego momentum jako punktu odniesienia przed okresem pomiaru późniejszych zwrotów. Warianty z makro i COT pozostają opcjonalne po MVP.
- [ ] Udostępnić przez Axum ranking, szczegóły spółki, zebrane serie danych i status aktualizacji.

## Etap 7. Frontend

- [ ] Najpierw dodać kartę spółki otwieraną kliknięciem wiersza rankingu. Przy każdym punkcie pokazać strzałkę: +1 w górę, −1 w dół, 0 jako kreskę, a `null` jako puste pole z krótkim powodem. Pod strzałką pokazać surowe składniki i przeliczoną relację, z której powstał punkt: kwoty raportu obok marży, kapitału i długu; ostatni `close` i SMA200 przy trendzie; dwie sumy shortów przy sentymencie. Financials nie dostaje strzałek dla wskaźników, których jego szablon nie liczy.
- [ ] Następnie dodać wykres spółki: świece OHLC z zapisanej historii cen, a pod spodem słupki `final_score` wyłącznie z dni, w których `rankr` zapisał ranking. Nie odtwarzać punktów za lata przed startem archiwum rankingów. Historia cen może być wieloletnia; słupki oceny obejmują tylko zapisane rankingi.
- [ ] Potem dodać widok rejestru KNF dla spółki, nie nazywany COT: tabelę posiadaczy z procentem i datą oraz wykres sumy ujawnionych shortów w czasie. Pusta lista zostaje pusta.

## Etap 8. Walidacja na zebranych danych

- [ ] Porównać zapisane rankingi z późniejszymi zmianami cen do lutego 2027. Dane fundamentalne ograniczyć do obserwacji zgromadzonych od uruchomienia importera; dla cen korzystać także z pobranego archiwum EOD, używając do sygnałów wyłącznie historii dostępnej na dzień danego rankingu.
- [ ] Dla ocen z pełnymi 3 miesiącami obserwacji policzyć korelację Spearmana i różnicę zwrotów najwyżej oraz najniżej ocenionych spółek; porównać ustalone modele.
- [ ] Wygenerować tabele i wykresy w Rust oraz opisać pokrycie danych, długość obserwacji i wyniki. Ocenę po 6 i 12 miesiącach pozostawić na dalsze zbieranie danych.

## Etap 9. Ukończenie pracy

- [ ] Sprawdzić cały przepływ: automatyczny import -> baza -> scoring -> API -> frontend.
- [ ] Przygotować instrukcję lokalnego uruchomienia i zaktualizować README oraz AI_CONTEXT.md.
- [ ] Ukończyć rozdziały pracy, wygenerować finalny PDF i przygotować prezentację aplikacji oraz wyników walidacji.
