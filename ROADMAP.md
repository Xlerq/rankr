# rankr — roadmap

## Cel

Lokalna aplikacja do rankingu pełnego WIG20 według potencjału wzrostu w horyzoncie 3–12 miesięcy. Scoring: fundamenty i wycena + trend + makro sektorowe + COT dla wybranych spółek.

Stos: Rust, Leptos/WASM, Axum/Tokio, SurrealDB, Plotters. Import, automatyzacja i analiza również w Rust.

Dane zbieramy od uruchomienia importera do lutego 2027, bez pobierania archiwów. Na start zapisujemy bieżące ceny, skład WIG20 i najnowsze dostępne raporty; potem kolejne aktualizacje.

Po każdym etapie uzupełniamy odpowiadającą mu część `thesis/main.tex` i testujemy wykonany moduł.

## Etap 0. Źródła danych

- [ ] Wybrać źródła bieżących cen, składu WIG20 i fundamentów dla banków, ubezpieczycieli oraz spółek niefinansowych.
- [ ] Wybrać serie makro oraz rynki COT powiązane ze spółkami WIG20.
- [ ] Sprawdzić dostępne pola, daty publikacji, częstotliwość aktualizacji i warunki wykorzystania danych.

## Etap 1. Podstawa projektu i baza danych

- [ ] Utworzyć workspace Rust i skonfigurować połączenie z SurrealDB.
- [ ] Przygotować schemat i migracje dla instrumentów, składu indeksu, cen, fundamentów, makro, COT oraz wyników scoringu.
- [ ] Zapisywać źródło, datę publikacji i pobrania danych; zachowywać kolejne wersje obserwacji.

## Etap 2. Automatyczny import i rozpoczęcie zbierania danych

- [x] Przygotować importer i parser podstawowych fundamentów GPW/Notoria w Rust, z bieżącą listą WIG20 i zapisem surowych oraz odczytanych danych do JSON (`importer/`).
- [ ] Zaimplementować import wszystkich wybranych źródeł w Rust i zastąpić dotychczasowe skrypty Python/Bash.
- [ ] Uruchomić harmonogram: ceny po sesji, fundamenty i makro po publikacji, COT co tydzień z obsługą opóźnień.
- [ ] Dodać walidację, ochronę przed duplikatami, ponowienia i rejestrowanie błędów oraz przerw w zbieraniu danych.
- [ ] Pozostawić importer działający podczas budowy pozostałych modułów; regularnie wykonywać kopię zebranych danych.

## Etap 3. Scoring fundamentalny i techniczny

- [ ] Zaimplementować wskaźniki fundamentów i wyceny dla banków, ubezpieczycieli i spółek niefinansowych.
- [ ] Dodać wskaźniki trendu oparte na zgromadzonych cenach; oznaczać niewystarczającą długość serii.
- [ ] Dodać normalizację do skali 0–100, konfigurowalne wagi, wkłady wskaźników i oznaczenia brakujących danych.

## Etap 4. Makro sektorowe

- [ ] Zdefiniować wpływ stóp procentowych, inflacji, aktywności gospodarczej i wybranych kursów walut na spółki lub sektory.
- [ ] Zaimplementować reguły dodające lub odejmujące punkty zależnie od otoczenia makro.
- [ ] Włączyć komponent makro do scoringu i zapisywać przyczyny jego punktacji.

## Etap 5. Wpływ COT na scoring

- [ ] Obliczać pozycję netto wybranych kategorii uczestników i jej zmianę między zebranymi raportami.
- [ ] Przypisać rynki COT do odpowiednich spółek i ustalić reguły punktacji.
- [ ] Włączyć komponent COT do scoringu i zapisywać jego wkład oraz datę użytego raportu.

## Etap 6. Backend i zapisywanie rankingów

- [ ] Przeliczać i zapisywać ranking po aktualizacji danych, razem z wersją konfiguracji i składowymi wyniku.
- [ ] Ustalić konfiguracje do walidacji: fundamenty z wyceną, warianty dodające trend, makro i COT oraz proste momentum; zapisywać ich wyniki przed okresem pomiaru późniejszych zwrotów.
- [ ] Udostępnić przez Axum ranking, szczegóły spółki, zebrane serie danych i status aktualizacji.

## Etap 7. Frontend

- [ ] Zbudować ranking WIG20 w Leptos z wyborem zapisanej daty, sortowaniem i filtrowaniem sektorów.
- [ ] Dodać kartę spółki ze składowymi wyniku, wskaźnikami, źródłami i datami danych.
- [ ] Dodać wykresy Plotters dla cen, scoringu, makro i COT oraz obsługę braków danych i błędów aktualizacji.

## Etap 8. Walidacja na zebranych danych

- [ ] Porównać zapisane rankingi z późniejszymi zmianami cen, korzystając wyłącznie z danych zgromadzonych od uruchomienia importera do lutego 2027.
- [ ] Dla ocen z pełnymi 3 miesiącami obserwacji policzyć korelację Spearmana i różnicę zwrotów najwyżej oraz najniżej ocenionych spółek; porównać ustalone modele.
- [ ] Wygenerować tabele i wykresy w Rust oraz opisać pokrycie danych, długość obserwacji i wyniki. Ocenę po 6 i 12 miesiącach pozostawić na dalsze zbieranie danych.

## Etap 9. Ukończenie pracy

- [ ] Sprawdzić cały przepływ: automatyczny import -> baza -> scoring -> API -> frontend.
- [ ] Przygotować instrukcję lokalnego uruchomienia i zaktualizować README oraz AI_CONTEXT.md.
- [ ] Ukończyć rozdziały pracy, wygenerować finalny PDF i przygotować prezentację aplikacji oraz wyników walidacji.
