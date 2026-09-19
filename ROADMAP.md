# rankr — roadmap

## Cel

Lokalna aplikacja webowa do rankingu spółek GPW według względnego potencjału wzrostu w horyzoncie 3–12 miesięcy. Wyższy wynik końcowy oznacza większą szansę na lepszy zwrot niż inne spółki z tego samego koszyka. Wynik nie jest oceną tego, jak dobra jest spółka, ani prognozą zwrotu w procentach. Aplikacja nie stanowi porady inwestycyjnej.

Koszyk MVP obejmuje pełne WIG20. Później możliwe będzie rozszerzenie na inne akcje GPW; 20 spółek nie jest stałym, ostatecznym wszechświatem. Projektowanie tego rozszerzenia pozostaje poza obecnym zakresem.

Widokiem głównym jest tabela spółek. Dla każdej spółki liczone są wyniki trzech rodzin, każdy z ustaloną wagą. Wynik końcowy to suma ważona wyników tych trzech rodzin, będąca miarą potencjału wzrostu i podstawą sortowania. Suma nie podlega normalizacji końcowej ani mapowaniu na oczekiwany procent zwrotu.

Trzy rodziny wyników v1: `fundamental` — najważniejsza, obejmująca kondycję finansową, wycenę i dynamikę wyników (wzrost przychodów i zysku, nie sam poziom ROE); `technical` — trend/momentum ceny i sezonowość miesięczna z wieloletniej historii dziennej; `sentiment` — sentyment, bez ustalonego źródła danych. v1 używa jednego zestawu wag dla całego WIG20, w tym banków. Sezonowość jest sygnałem wewnątrz `technical`, bez czwartej wagi; jej wzór pozostaje do opracowania w etapie scoringu.

Frontend, backend i silnik scoringu w pełni w Rust. Stos: Rust, Leptos/WASM, Axum/Tokio, SurrealDB, Plotters. Import, automatyzacja i analiza również w Rust.

Poza zakresem MVP: bot inwestycyjny, integracja z brokerem, dane realtime i uczenie maszynowe.

Dane zbieramy od uruchomienia importera do lutego 2027. Dla cen pobieramy także całą dostępną historię dzienną OHLCV z Yahoo Finance, potrzebną do trendu/momentum i sezonowości; zakaz pobierania archiwów nie dotyczy cen. Ostatnie kompletne dane dzienne spółek pochodzą z Yahoo Finance, a FX/złoto z TradingView. Fundamenty zbieramy wyłącznie jako bieżące dane GPW/Notoria: najnowsze dostępne raporty, potem kolejne aktualizacje, bez odtwarzania archiwum sprawozdań. Zapisujemy również bieżący skład WIG20.

Po każdym etapie uzupełniamy odpowiadającą mu część `thesis/main.tex` i testujemy wykonany moduł.

## Etap 0. Źródła danych

- [ ] Wybrać źródła bieżących cen, składu WIG20 i fundamentów dla banków, ubezpieczycieli oraz spółek niefinansowych.
- [ ] Opcjonalnie po MVP: wybrać serie makro oraz rynki COT powiązane ze spółkami WIG20.
- [ ] Sprawdzić dostępne pola, daty publikacji, częstotliwość aktualizacji i warunki wykorzystania danych.

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

- [ ] Zaimplementować wynik najważniejszej rodziny `fundamental` dla pełnego WIG20, obejmującej kondycję finansową, wycenę i dynamikę wyników; dynamika obejmuje wzrost przychodów i zysku, nie sam poziom ROE.
- [ ] Dodać wynik rodziny `technical` oparty na trendzie/momentum oraz sezonowości cen w analogicznych miesiącach z wieloletniej historii EOD z Yahoo Finance (docelowo około 12 lat lub więcej); oznaczać niewystarczającą długość serii dla danego sygnału. Sezonowość nie dodaje czwartej wagi.
- [ ] Uwzględnić wynik `sentiment` jako trzecią rodzinę; źródło danych pozostaje nieustalone.
- [ ] Obliczać potencjał wzrostu jako sumę ważoną wyników `fundamental`, `technical` i `sentiment`, bez normalizacji końcowej i mapowania na procent zwrotu. Stosować jeden zestaw wag dla całego WIG20, w tym banków, oraz prezentować wyniki rodzin i ich wagi.

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

- [ ] Przeliczać i zapisywać ranking względnego potencjału wzrostu po aktualizacji danych jako sumę ważoną składowych, razem z wersją konfiguracji, składowymi wyniku i ich wagami; stosować jeden zestaw wag dla całego WIG20, w tym banków.
- [ ] Zapisywać wyniki modelu v1 (`fundamental`, `technical`, `sentiment`) oraz prostego momentum jako punktu odniesienia przed okresem pomiaru późniejszych zwrotów. Warianty z makro i COT pozostają opcjonalne po MVP.
- [ ] Udostępnić przez Axum ranking, szczegóły spółki, zebrane serie danych i status aktualizacji.

## Etap 7. Frontend

- [ ] Zbudować w Leptos główny widok tabeli pełnego WIG20, domyślnie sortowanej według sumy ważonej składowych potencjału wzrostu, z wyborem zapisanej daty i filtrowaniem sektorów.
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
