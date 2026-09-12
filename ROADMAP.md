# rankr — roadmap

## Cel

Lokalna aplikacja do rankingu pełnego WIG20 według potencjału wzrostu w horyzoncie 3–12 miesięcy. Scoring: fundamenty i wycena + trend + makro sektorowe + COT dla wybranych spółek.

Stos: Rust, Leptos/WASM, Axum/Tokio, SurrealDB, Plotters. Import, automatyzacja i analiza również w Rust.

## Etap 0. Sprawdzenie danych

- [ ] Sprawdzić źródła cen, fundamentów, historycznego składu WIG20, makro i COT oraz warunki ich wykorzystania.
- [ ] Pobrać próbki dla banku, ubezpieczyciela i dwóch spółek niefinansowych.
- [ ] Sprawdzić dostępność dat publikacji i wybrać okres analizy historycznej.

## Etap 1. Pierwsze demo

- [ ] Utworzyć workspace Rust i połączyć Axum, SurrealDB oraz Leptos.
- [ ] Zaimportować próbki, policzyć prosty scoring dla czterech spółek i udostępnić ranking przez API.
- [ ] Wyświetlić ranking, składowe wyniku i wykres ceny w przeglądarce.

## Etap 2. Import i baza danych

- [ ] Zaimplementować w Rust import cen, fundamentów, składu WIG20 i makro; zastąpić dotychczasowe skrypty Python/Bash.
- [ ] Zapisywać źródła, daty publikacji, historyczne wersje danych i wyniki scoringu w SurrealDB.
- [ ] Objąć importem pełny WIG20 oraz dawnych uczestników indeksu potrzebnych do badania; dodać walidację i ochronę przed duplikatami.

## Etap 3. Scoring fundamentalny i techniczny

- [ ] Wybrać i zaimplementować wskaźniki fundamentów oraz wyceny dla banków, ubezpieczycieli i spółek niefinansowych.
- [ ] Dodać wskaźniki trendu, normalizację do skali 0–100 i konfigurowalne wagi.
- [ ] Zapisywać wkład każdego wskaźnika do wyniku oraz oznaczać brakujące i nieaktualne dane.

## Etap 4. Makro sektorowe

- [ ] Wybrać serie makro: stopy procentowe, inflację, aktywność gospodarczą i potrzebne kursy walut.
- [ ] Przypisać spółkom lub sektorom reguły wpływu tych danych na punktację.
- [ ] Włączyć komponent makro do rankingu i pokazywać jego wkład w ocenę spółki.

## Etap 5. Dane COT

- [ ] Zaimplementować import bieżących i historycznych raportów CFTC dla 1–2 wybranych rynków.
- [ ] Obliczać pozycję netto uczestników i jej zmianę tygodniową; ustalić reguły punktacji.
- [ ] Powiązać rynki COT z odpowiednimi spółkami i dodać osobno widoczny wkład COT do wyniku.

## Etap 6. Automatyczne aktualizacje

- [ ] Dodać harmonogram w Rust: ceny po sesji, fundamenty i makro po publikacji, COT co tydzień z obsługą przesunięć publikacji.
- [ ] Przeliczać ranking po imporcie nowych danych i nadrabiać zaległości po ponownym uruchomieniu aplikacji.
- [ ] Dodać ponowienia pobierania, status aktualizacji i zachowanie ostatniego poprawnego rankingu przy awarii.

## Etap 7. Pełny interfejs i API

- [ ] Dodać ranking WIG20 z wyborem daty, sortowaniem i filtrowaniem sektorów.
- [ ] Zbudować kartę spółki: składowe wyniku, wskaźniki, źródła i daty danych.
- [ ] Dodać wykresy ceny, historii scoringu, makro i COT oraz obsługę błędów i braków danych.

## Etap 8. Analiza historyczna

- [ ] Ustalić wagi i modele przed końcowym testem: fundamenty z wyceną, następnie warianty dodające trend, makro i COT; punkt odniesienia: prosty ranking momentum.
- [ ] Odtworzyć rankingi z ówczesnego składu WIG20 i danych dostępnych w dniu oceny; oddzielić okres doboru parametrów od późniejszego testu.
- [ ] Porównać rankingi z późniejszymi zwrotami po 3, 6 i 12 miesiącach: korelacja Spearmana i różnica zwrotów najwyżej oraz najniżej ocenionych spółek.
- [ ] Wygenerować tabele i wykresy w Rust oraz opisać wyniki i ograniczenia badania.

## Etap 9. Testy, dokumentacja i wydanie

- [ ] Uzupełnić testy importerów, scoringu, bazy, API i automatyzacji; uruchamiać kontrole Rust/WASM w CI.
- [ ] Przygotować jedno polecenie uruchamiające lokalne demo bez sieci i instrukcję aktualizacji danych rzeczywistych.
- [ ] Uaktualnić README i AI_CONTEXT.md; ukończyć `thesis/main.tex`, wygenerować PDF i przygotować demonstrację projektu.
