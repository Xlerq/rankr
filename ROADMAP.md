# rankr — roadmap pracy inżynierskiej

Aktualizacja: 12 września 2026. Zakres ustalony na podstawie wizji autora. Etapy określają zależności i rezultaty; nie są deklaracją terminów przy nieustalonym budżecie czasu.

## Cel i ustalony zakres

Zbudować lokalną aplikację webową, która szereguje spółki WIG20 według ocenianego potencjału wzrostu kursu w horyzoncie 3–12 miesięcy. Wynik łączy fundamenty z wyceną, trend, otoczenie makroekonomiczne oraz pozycjonowanie COT dla spółek z uzasadnioną ekspozycją na raportowany rynek.

Wysoki wynik oznacza korzystną sytuację według modelu. Nie jest prognozą konkretnego procentowego wzrostu ani prawdopodobieństwem zysku. Zdolność rankingu do porządkowania przyszłych stóp zwrotu podlega badaniu, a nie założeniu.

Wersja na inżynierkę obejmuje:

- Pełny aktualny WIG20 w aplikacji oraz historyczny skład indeksu w wybranym okresie badania, również spółki, które później opuściły indeks.
- Ranking, widok spółki, rozbicie punktacji, źródła danych i historię wyników.
- Profile fundamentalne dla banków, ubezpieczycieli i spółek niefinansowych, z ograniczoną liczbą uzasadnionych wskaźników.
- Makro aktywnie wpływające na ocenę zależnie od ekspozycji spółki lub sektora.
- Automatyczny cotygodniowy import COT i jego jawny wpływ na punktację wybranych spółek.
- Automatyczne aktualizacje pozostałych danych zgodnie z ich częstotliwością i dostępnością.
- Porównanie ustalonych modeli z prostym rankingiem odniesienia na danych dostępnych w chwili oceny, czyli point-in-time.
- Odtwarzalne demo lokalne działające bez sieci, niezależne od trybu aktualizacji danych rzeczywistych.

Cały kod aplikacji, importu, scoringu, harmonogramu, analityki i wykresów powstaje w Rust. Stos: Leptos/WASM, Axum/Tokio, SurrealDB, Plotters. SurrealQL, LaTeX i deklaratywna konfiguracja narzędzi pozostają właściwymi formatami dla bazy, pracy i budowania. Python i R nie są zależnościami wersji końcowej; własna logika projektu w Bash również zostaje zastąpiona poleceniami Rust.

Poza zakresem: pozostałe indeksy jako osobne uniwersum produktu, intraday, real-time, ML, analiza wiadomości, dobór portfela, pełny symulator transakcyjny, brokerzy, wykonywanie zleceń, publiczny hosting, konta, płatności i alerty wysyłane użytkownikom. Dane o dawnych uczestnikach WIG20 są częścią badania, a nie rozszerzeniem produktu na cały rynek.

## Stan początkowy i zasady realizacji

Stan stwierdzony w repozytorium przy aktualizacji roadmapy:

- [x] Struktura katalogów oraz szkielet pracy w `thesis/main.tex`.
- [x] Próbki Stooq, GPW Benchmark, GPW/Notoria i NBP oraz wstępne mapowanie instrumentów.
- [x] Pliki schematu, seeda i skryptów weryfikacyjnych SurrealDB. Dokumentacja opisuje wcześniejszy import; ta aktualizacja roadmapy nie powtarza testu bazy.
- [ ] Workspace i implementacja Rust. Brak `Cargo.toml` i plików `.rs`; `backend/`, `frontend/` i `shared/` zawierają tylko `.gitkeep`.
- [ ] Importery produkcyjne, scoring, API, interfejs, automatyzacja i badanie historyczne.

Wcześniejszy wpis o ukończonym `/api/health` nie odpowiada aktualnemu checkoutowi. `AI_CONTEXT.md`, README, dokumentacja bazy i tekst pracy wymagają uzgodnienia z nowym zakresem, m.in. usunięcia założeń o R i wyłączonym makro.

Zasady pracy:

1. Najpierw sprawdzać dane i ryzyka metodyczne, następnie rozbudowywać funkcjonalność.
2. Każdy etap kończyć działającym rezultatem i dowodem spełnienia kryteriów odbioru.
3. Pierwszy wycinek demonstracyjny projektować jako cel na około dwa tygodnie pracy, bez obietnicy kalendarzowej. W razie przeciążenia ograniczyć liczbę metryk i wygląd demo.
4. Testy i tekst pracy powstają razem z danym modułem. Końcowy etap służy integracji i redakcji.
5. Zadania implementacyjne rozbijać na bilety do około jednej godziny z pojedynczym rezultatem, np. parser jednej tabeli wraz z fixture'em.
6. Nie dodawać nowego źródła lub wskaźnika bez wskazania, jaką hipotezę albo brak danych rozwiązuje.
7. Leptos i SurrealDB oznaczają koszt poznania integracji i narzędzi. Sprawdzić ich wspólny przepływ w pierwszym demo; nie odkładać tego ryzyka za budowę całego silnika.

## Etapy

### Etap 0. Audyt danych i wykonalności badania

Cel: ustalić, co rzeczywiście można policzyć, zanim model zacznie wymagać nieosiągalnych danych.

- [ ] Sporządzić w pracy macierz dostępności: źródło, instrument/profil, pola, jednostki, zakres historii, data publikacji, rewizje, automatyczny dostęp i warunki wykorzystania. Istniejąca próbka nie potwierdza pełnego pokrycia.
- [ ] Sprawdzić fundamenty i wycenę dla banku, ubezpieczyciela oraz dwóch spółek niefinansowych, w tym jednej z możliwym powiązaniem COT. Zweryfikować przynależność wybranych spółek do właściwego składu indeksu.
- [ ] Dla każdego profilu znaleźć kilka kolejnych historycznych raportów, w tym raport roczny, śródroczny i korektę, jeśli występuje. Sprawdzić daty publikacji i rozróżnienie danych skonsolidowanych oraz jednostkowych.
- [ ] Sprawdzić dostęp do liczby akcji lub kapitalizacji potrzebnej do wyceny na daną datę, historii składu WIG20, cen, splitów i zdarzeń kończących notowania.
- [ ] Ocenić Stooq, GPW Benchmark i GPW/Notoria jako kandydatów, a nie gwarantowane źródła. W razie braków sprawdzić raporty emitentów i oficjalne publikacje. Nie budować od razu uniwersalnego parsera PDF.
- [ ] Zweryfikować minimalne serie makro: stopę referencyjną NBP, inflację i miarę aktywności gospodarczej z oficjalnych publikacji; FX dołączyć przy konkretnej ekspozycji. Próbki NBP z kursami i złotem nie wystarczają do oceny cyklu gospodarczego.
- [ ] Pobrać próbki oficjalnych raportów CFTC dla 1–2 rynków, sprawdzić kody kontraktów, kategorie uczestników, historię i możliwość odtworzenia momentu publikacji. Kandydat do pierwszego testu: miedź i ekspozycja KGHM.
- [ ] Ustalić wspólny okres badania na podstawie pokrycia. Cel roboczy: około pięciu lat dat rankingowych, z dodatkową historią do wskaźników i późniejszymi cenami do oceny zwrotów. Krótszy zakres opisać jako ograniczenie; liczne nakładające się obserwacje nie rekompensują krótkiej historii.
- [ ] Zapisać znaczenie score'u, granice projektu i dostępność danych w `thesis/main.tex`; uzgodnić z nimi `AI_CONTEXT.md`, README i dokumentację bazy.

Rezultat: macierz danych, rzeczywiste próbki trzech profili i COT oraz wybrany zakres historyczny.

Warunek ukończenia: dla wymaganych metryk istnieje sprawdzona ścieżka pozyskania danych i dat dostępności. Luki mają konkretną politykę, nie założenie, że zostaną uzupełnione później.

Warunek ograniczenia zakresu: najpierw zmniejszyć liczbę metryk lub badany okres. Dopuszczalny jest ograniczony, ręcznie zweryfikowany zbiór historyczny importowany przez Rust, jeśli warunki użycia na to pozwalają. Bieżące aktualizacje pozostają automatyczne. Jeżeli nawet taki zbiór nie pozwala na rzetelne badanie, opisać konflikt z wymaganiem i uzgodnić zmianę — nie zastępować po cichu point-in-time danymi pobranymi dzisiaj.

### Etap 1. Pierwszy działający przepływ przez całą aplikację

Zależność: reprezentatywne próbki i podstawowe ustalenia z etapu 0. Nie trzeba czekać na pobranie całej historii.

- [ ] Utworzyć minimalny workspace: współdzielona domena i DTO, backend/CLI oraz frontend. Osobne craty wydzielać przy rzeczywistej potrzebie, bez projektowania frameworka.
- [ ] Przypiąć toolchain Rust, zależności i wersję SurrealDB; sprawdzić kompilację backendu oraz Leptos/WASM.
- [ ] Zaimplementować importer Rust dla zamrożonego zbioru 3–4 spółek obejmującego wszystkie trzy profile.
- [ ] Zapisać dane w SurrealDB i policzyć demonstracyjny wynik z 2–3 dostępnych wskaźników na profil. Konfigurację oznaczyć jako prototypową, nie używać jej do wniosków badawczych.
- [ ] Wystawić ranking i dane jednej spółki przez Axum; pokazać je w Leptos wraz z wyjaśnieniem wkładu wskaźników.
- [ ] Wyświetlić pierwszy wykres ceny w Plotters oraz stany braku danych i błędu.
- [ ] Dodać podstawowe CI: formatowanie, Clippy, testy Rust i sprawdzenie kompilacji frontendu.
- [ ] Przygotować polecenie demonstracyjne w Rust i test przepływu `fixture -> baza -> score -> API`.

Rezultat: lokalne demo, w którym można przejść od pozycji w rankingu do danych uzasadniających wynik.

Warunek ukończenia: przepływ działa dla wszystkich wybranych profili bez Python/R i bez dostępu do dostawców danych. To wycinek funkcjonalny, jeszcze bez pełnego makro, COT i historii WIG20.

Warunek ograniczenia zakresu: uprościć wygląd i metryki. Zachować przepływ przez bazę, backend i frontend oraz różnorodność profili.

### Etap 2. Dane point-in-time i odtwarzalny import w Rust

Zależność: etapy 0–1. Rozszerzać model na podstawie sprawdzonych danych.

- [ ] Rozdzielić instrument, historię członkostwa, ceny, raport finansowy, obserwację makro/COT, wersję źródła i przebieg obliczenia score'u. Zachować identyfikatory dostawców oraz historię zmian symboli.
- [ ] Rozróżnić `period_end`/`observed_at`, `published_at`, `available_at` i `fetched_at`. Pierwsze pola opisują okres obserwacji i publikację, `available_at` granicę dopuszczalnego użycia, a `fetched_at` lokalne pobranie. Zapisać pochodzenie i pewność dat.
- [ ] Wybierać dla daty odcięcia tylko wersje dostępne do tego momentu. Obsłużyć rewizje fundamentów i makro bez zastępowania nimi wcześniejszej wiedzy.
- [ ] Przenieść sprawdzone importery Stooq, GPW Benchmark, GPW/Notoria i NBP do poleceń Rust; źródła wymagające zastępstwa obsłużyć zgodnie z audytem.
- [ ] Walidować jednostki, waluty, okresy narastające i pojedyncze kwartały, raporty skonsolidowane, braki i chronologię. Unikać podwójnego liczenia danych przy budowie TTM.
- [ ] Rozdzielić ceny używane do historycznej wyceny od serii przystosowanych do liczenia zwrotów. Nie mieszać cen skorygowanych o późniejsze zdarzenia z historyczną liczbą akcji.
- [ ] Zapisywać surowy artefakt, hash, wersję parsera i wynik importu; zapewnić idempotencję, timeouty, ograniczone ponowienia oraz kontrolowane zatwierdzanie kompletnej partii.
- [ ] Wprowadzić migracje i testy fixture'ów parserów, zapytań według daty dostępności, duplikatów, błędnych świec oraz rewizji.

Rezultat: ten sam zamrożony zbiór można zaimportować ponownie bez duplikatów i odtworzyć stan wiedzy dla wskazanej daty.

Warunek ukończenia: test wykazuje, że późniejszy raport lub korekta nie zmienia wcześniejszego rankingu. Nieznana data publikacji nie jest automatycznie zastępowana końcem okresu sprawozdawczego. Dane syntetyczne służą wyłącznie demonstracji/testom, nigdy ocenie skuteczności.

### Etap 3. Fundamenty, wycena i trend dla pełnego WIG20

Zależność: model czasu i import z etapu 2.

- [ ] Wybrać mały zestaw metryk na podstawie uzasadnienia ekonomicznego i pokrycia danych. Kandydaci: rentowność, zadłużenie, jakość przepływów, wzrost i wycena; dla banków i ubezpieczycieli właściwe im miary rentowności, ryzyka, kapitału i wyceny.
- [ ] Zdefiniować wzory, znaki, jednostki, zasady dla ujemnych zysków/kapitału, zerowych mianowników i wartości odstających. Wskaźnik niemający sensu ekonomicznego nie otrzymuje punktów za skrajną wartość.
- [ ] Zbudować normalizację i agregację w czystych funkcjach Rust. Progi ustalić z uzasadnienia albo na okresie rozwojowym; nie korzystać ze statystyk przyszłego zbioru testowego.
- [ ] Opisać porównywalność profili. Nie stosować percentyla dla grupy złożonej z jednej spółki. Sam przedział `0–100` nie dowodzi, że 80 punktów banku odpowiada 80 punktom firmy przemysłowej.
- [ ] Dodać ograniczony zestaw cech trendu. Jako prosty ranking odniesienia przyjąć roboczo momentum 12–1, czyli zmianę ceny z poprzednich 12 miesięcy z pominięciem ostatniego miesiąca. Dokładne okna ustalić przed badaniem.
- [ ] Zdefiniować wersjonowaną konfigurację wag bez narzuconych proporcji 65/25/10. Rozdzielić konfigurację prototypu, rozwojową i zamrożoną do badania.
- [ ] Pokazywać osobno kompletność i świeżość danych. Brak krytycznej metryki oznacza niepełną ocenę; pozostałe braki obsługuje jedna jawna polityka. Braków nie wykorzystywać do sztucznego zwiększania wyniku przez dowolne przeważanie dostępnych metryk.
- [ ] Rozszerzyć obsługę na pełny bieżący WIG20 i historyczne spółki z okresu badania. Każdy uczestnik jest widoczny, z wynikiem lub jednoznacznym powodem niepełnej oceny; raportować pokrycie, nie ukrywać trudnych przypadków.
- [ ] Zapisywać składowe, wkłady, użyte rekordy, datę odcięcia, wersję konfiguracji, wersję kodu i identyfikator zbioru danych; przetestować deterministyczność i przypadki brzegowe.

Rezultat: ranking bazowy z fundamentów, wyceny i trendu oraz karta wyjaśniająca wynik każdej obsługiwanej spółki.

Warunek ukończenia: wszystkie profile i cały WIG20 są obsłużone, pokrycie metryk jest zmierzone, a suma wkładów odtwarza wynik przed zaokrągleniem. Brak wymaganych danych pozostaje jawnym brakiem, nie zaliczonym zadaniem importu.

### Etap 4. Makro wpływające na punktację

Zależność: ranking bazowy z etapu 3 i datowane serie makro z etapu 2.

- [ ] Ograniczyć pierwszą wersję do kilku sprawdzonych serii i reguł. Dla każdej zapisać mechanizm, ekspozycję, oczekiwany kierunek, ograniczenie siły wpływu i moment dostępności.
- [ ] Rozróżnić poziom stóp i ich zmianę. Nie przyjmować bezwarunkowo, że im wyższe stopy, tym lepszy bank albo że niższe stopy pomagają wszystkim pozostałym. Uwzględnić przeciwstawne kanały lub jawnie ograniczyć zakres reguły.
- [ ] Zdefiniować mapowanie sektorowe z możliwością uzasadnionego wyjątku dla spółki. Nie dopisywać jednakowego bonusu wszystkim uczestnikom, ponieważ nie zmieni ich kolejności.
- [ ] Traktować korzystny wpływ jako premię względem neutralnego kontekstu. Roboczo niekorzystny wpływ obniża komponent względem neutralnego; siła i granice wymagają zapisanej metodyki.
- [ ] Wprowadzić daty obowiązywania mapowania ekspozycji. Dzisiejszej struktury biznesu nie przypisywać automatycznie spółce sprzed kilku lat.
- [ ] Wyświetlać obserwację, uruchomioną regułę i wkład makro; kontrolować powielanie informacji już obecnej w fundamentach lub trendzie.
- [ ] Dodać testy reguł i przełącznik wyłączenia makro do eksperymentu. Dla porównań zachowywać pozostałe parametry modelu.

Rezultat: zmiana obserwacji makro może zmienić ranking spółek o różnych ekspozycjach, z możliwym do odtworzenia uzasadnieniem.

Warunek ukończenia: reguły mają jawne źródła danych i uzasadnienia, a punkty pojawiają się dopiero od dostępności publikacji. Hipoteza o ich skuteczności zostanie sprawdzona w badaniu.

### Etap 5. COT jako mierzalny składnik rankingu

Zależność: audyt COT, model czasu oraz mechanizm ekspozycji z etapów 0, 2 i 4.

- [ ] Zaimplementować importer oficjalnych danych CFTC w Rust. Wybrać właściwy typ raportu i wariant `futures only` albo `futures and options combined`; nie sumować nakładających się raportów.
- [ ] Identyfikować rynek przez stabilny kod i typ raportu. Dla surowców rozważyć Disaggregated; TFF wykorzystywać tylko przy uzasadnionym rynku finansowym. Nie utożsamiać kategorii uczestników między raportami.
- [ ] Zacząć od 1–2 rynków powiązanych z konkretnymi spółkami. Miedź/KGHM jest kandydatem do weryfikacji, a nie automatycznie zaakceptowaną regułą.
- [ ] Obliczać niewielki zestaw cech: pozycję netto wybranej kategorii względem open interest, zmianę tygodniową oraz położenie względem własnej wcześniejszej historii. Okna i kategorię ustalić przed końcowym badaniem; obsłużyć zerowy open interest, luki i rozgrzewkę wskaźnika.
- [ ] Zapisać hipotezę interpretacji: np. kontynuacja pozycjonowania albo przeciążenie jednej strony rynku. Nie wybierać interpretacji po obejrzeniu wyników testowych i nie zakładać, że duży uczestnik musi mieć rację.
- [ ] Dodać osobno widoczny, ograniczony wkład `cot_score`. Spółka bez uzasadnionej ekspozycji otrzymuje neutralny wkład COT. Brak raportu dla spółki z ekspozycją jest odrębnym stanem jakości danych.
- [ ] Zachować datę obserwacji i publikacji. Raportu opisującego wtorek nie używać we wtorkowym rankingu, jeśli opublikowano go w piątek. Dla niepewnych historycznych terminów ustalić udokumentowaną politykę i zakres wyłączeń.
- [ ] Przetestować duplikat raportu, przesuniętą publikację, brak rynku/kategorii, nieaktualność i przełącznik wyłączenia COT przy niezmienionych pozostałych parametrach.

Rezultat: rzeczywiste dane COT są importowane i wpływają na wybrane wyniki, a użytkownik widzi powód tego wpływu.

Warunek ukończenia: przynajmniej jedna uzasadniona ekspozycja działa przez cały przepływ, a użycie COT można odtworzyć i wyłączyć w porównaniu modeli. Jeżeli audyt nie potwierdzi sensownego powiązania, nie wymyślać go dla zaliczenia funkcji — jawnie wrócić do decyzji o jej zakresie.

CFTC zwykle publikuje raport w piątek o 15:30 czasu `America/New_York`, opisując poprzedni wtorek; harmonogram może się zmieniać. Korzystać z [opisu danych i API CFTC](https://www.cftc.gov/MarketReports/CommitmentsofTraders/index.htm) oraz [harmonogramu publikacji](https://www.cftc.gov/MarketReports/CommitmentsofTraders/ReleaseSchedule/index.htm). Stała godzina polska i założenie, że publikacja zawsze następuje w piątek, nie wystarczą.

### Etap 6. Automatyczne aktualizacje lokalne

Zależność: działające importery i scoring z etapów 2–5.

- [ ] Dodać polecenie aktualizacji oraz tryb cykliczny procesu Rust/Tokio. Harmonogram obejmuje tylko zadania potrzebne aplikacji; nie budować uniwersalnej platformy zadań.
- [ ] Aktualizować ceny po dostępności sesji dziennej, fundamenty i makro sprawdzać według ich publikacji, a COT według kalendarza z ponowieniami w ograniczonym oknie. Respektować limity źródeł.
- [ ] Zapisywać trwały stan ostatniego udanego zadania i nadrabiać brakujące publikacje po uruchomieniu komputera/aplikacji. Lokalny automat nie pobiera danych, gdy proces nie działa.
- [ ] Oddzielić udany import od przeliczenia rankingu. Przeliczać po zatwierdzeniu danych, bez publikowania częściowo zapisanych wyników.
- [ ] Wprowadzić ograniczone ponowienia, blokadę nakładających się przebiegów, log i status ostatniej próby/sukcesu. Przy awarii zachować poprzedni ranking z oznaczeniem jego wieku.
- [ ] Rozróżniać brak nowej publikacji, awarię pobierania i przeterminowanie danych. Nie zerować punktów COT tylko dlatego, że raport nie pojawił się o oczekiwanej godzinie; stosować ustalony termin ważności.
- [ ] Przetestować restart po przerwanym imporcie, ponowienie tego samego raportu, awarię sieci i nadrabianie zaległości z kontrolowanym zegarem lub ręcznym wywołaniem zadania.

Rezultat: aplikacja samodzielnie utrzymuje aktualne dane i wyniki podczas działania, a po przerwie nadrabia zaległości.

Warunek ukończenia: testowana awaria nie uszkadza ostatniego kompletnego stanu, nie tworzy duplikatów i jest widoczna w statusie aktualizacji. Cotygodniowy import COT nie wymaga ręcznego uruchamiania.

### Etap 7. Pełny interfejs użytkownika i kontrakty API

Zależność: rozwijać istniejący interfejs wraz z etapami 3–6; tutaj domknąć cały przepływ.

- [ ] Dopracować ranking całego WIG20: wynik, zmiana pozycji, komponenty, sektor/profil, data oceny, kompletność i świeżość. Rozróżniać niepełną ocenę od niskiego wyniku.
- [ ] Udostępnić najnowszy ranking i historyczne daty z rzeczywistym pokryciem; nie sugerować dostępności dowolnej daty bez danych.
- [ ] Zbudować kartę spółki: fundamenty i wycena, trend, reguły makro, COT, wkłady do wyniku oraz źródła i daty publikacji.
- [ ] Pokazać wykres ceny, historii score'u i odpowiednich serii makro/COT; oddzielić skale, żeby zestawienie nie sugerowało nieistniejącej korelacji.
- [ ] Dodać sortowanie i filtrowanie sektorów, obsługę ładowania, pustych wyników, błędów i nieaktualności oraz prosty responsywny układ.
- [ ] Ustabilizować DTO i endpointy: health/readiness, ranking z datą i konfiguracją, szczegóły spółki, ceny, historia wyników, użyte makro/COT i status aktualizacji.
- [ ] Sprawdzić kontrakty API i scenariusz użytkownika: ranking -> spółka -> przyczyna punktacji -> źródło. Techniczne hashe i identyfikatory przebiegów umieścić w szczegółach, nie na głównym ekranie.

Rezultat: lokalna aplikacja pozwala ocenić, dlaczego spółka zajmuje określone miejsce i na ile aktualne są podstawy tej oceny.

Warunek ukończenia: pełny scenariusz działa także przy niepełnych danych i awarii aktualizacji, bez ręcznego przeglądania bazy lub logów.

### Etap 8. Zamrożone porównanie modeli na danych historycznych

Zależność: dane i warianty scoringu z etapów 2–5. Protokół przygotować po audycie, a próbny przebieg wykonać przed dopracowaniem UI. Końcowe wyniki policzyć po zamrożeniu konfiguracji.

| Wariant | Składniki | Cel porównania |
| --- | --- | --- |
| B0 | Proste momentum 12–1 | Czy złożony ranking wnosi coś ponad prostą regułę cenową? |
| M1 | Fundamenty i wycena | Punkt wyjścia modelu wieloczynnikowego |
| M2 | M1 + trend | Wkład danych cenowych |
| M3 | M2 + makro sektorowe | Wkład otoczenia gospodarczego |
| M4 | M3 + COT | Dodatkowy wkład pozycjonowania |

- [ ] Z góry zapisać wzory, parametry, wagi i politykę braków. Przy wyłączaniu komponentów pozostawiać pozostałe parametry bez zmian; użycie stałej wartości neutralnej dla wyłączonego komponentu pozwala zachować skalę i izolować zmianę. Nie dostrajać każdego wariantu osobno na zbiorze testowym.
- [ ] Przyjąć roboczo 6 miesięcy jako główny horyzont oraz 3 i 12 jako pomocnicze. Przyjąć miesięczne daty oceny, chociaż aplikacja aktualizuje się częściej. Te decyzje i dokładne daty zamrozić przed oceną.
- [ ] Podzielić czas chronologicznie na część rozwojową i późniejszy końcowy test. Żaden przyszły zwrot użyty do doboru modelu nie może wchodzić w okres testowy; dla horyzontu 12 miesięcy może to wymagać pominięcia końcowych 12 miesięcy dat rankingowych części rozwojowej.
- [ ] Generować ranking z ówczesnego składu WIG20, raportów, rewizji, mapowania ekspozycji i cech dostępnych do daty odcięcia. Wspólne normalizacje liczyć tylko na dopuszczalnych danych.
- [ ] Zdefiniować początek pomiaru zwrotu po odcięciu informacji, np. na otwarciu następnej sesji, oraz koniec po ustalonej liczbie miesięcy z jawną regułą dni bez sesji. Nie przypisywać ceny zamknięcia sprzed publikacji jako ceny wejścia po tej publikacji.
- [ ] Jako główny wynik mierzyć wzrost kursu skorygowany o splity, bez dywidend, zgodnie z celem projektu. Jednolicie opisać fuzje, wycofania z obrotu i brak ceny końcowej; nie usuwać automatycznie spółek, które przestały być notowane. Zwrot całkowity z dywidendami jest opcjonalnym rozszerzeniem przy sprawdzonych danych.
- [ ] Policzyć korelację rang Spearmana z późniejszym zwrotem oraz różnicę średnich zwrotów najwyższego i najniższego kwartyla. Przy pełnych 20 ocenach są to grupy po 5 spółek; reguły remisów, braków i minimalnego pokrycia ustalić wcześniej.
- [ ] Porównywać modele na zgodnych datach i zbiorach spółek; publikować pokrycie i wyniki wykluczeń. Wkład COT ocenić również na z góry zdefiniowanej grupie spółek z ekspozycją, bez wyboru grup po wynikach.
- [ ] Zachowanie WIG20 traktować jako kontekst rynkowy, nie zamiennik rankingu odniesienia. Zachować zgodność wersji cenowej/dochodu całkowitego z mierzoną stopą zwrotu.
- [ ] Uwzględnić zależność między spółkami i nakładanie się horyzontów: dla niepewności użyć np. bootstrapu blokowego po datach z całymi przekrojami spółek i blokami dobranymi do horyzontu. Przy zbyt małej liczbie niezależnych okresów ograniczyć wnioski do opisowych.
- [ ] Analizę wrażliwości wag i braków wykonać na części rozwojowej. Raportować wszystkie zaplanowane warianty i horyzonty, również ujemne wyniki. Nie wybierać najlepszego z kilkunastu porównań jako dowodu sukcesu.
- [ ] Wygenerować tabele CSV/JSON i wykresy Plotters jednym poleceniem Rust z manifestu zbioru, wersji konfiguracji, kodu i ziarna losowego tam, gdzie jest potrzebne.

Rezultat: odtwarzalne porównanie B0 i M1–M4 z opisem pokrycia, niepewności i ograniczeń.

Warunek ukończenia: wynik można odtworzyć, nie zawiera informacji z przyszłości i rozdziela hipotezy od wniosków. Różnica zwrotów grup nie jest przedstawiana jako wynik wykonalnej strategii netto — nie modelujemy tu kosztów, płynności ani obrotu portfelem. Brak przewagi nad B0 nie oznacza nieukończenia pracy; oznacza brak potwierdzenia wartości prognostycznej modelu.

### Etap 9. Odtwarzalne wydanie lokalne i gotowa praca

Zależność: zamknięte funkcje aplikacji oraz raport badawczy. Rozdziały pracy są uzupełniane już w poprzednich etapach.

- [ ] Zapewnić jedno udokumentowane polecenie Rust uruchamiające demo z zamrożonych danych w osobnej bazie. Demo nie nadpisuje danych rzeczywistych; wymagania wstępne i wersje narzędzi są jawne.
- [ ] Oddzielić tryby demo, aktualizacji danych i badania historycznego. Podać polecenia, oczekiwane wyniki oraz sposób bezpiecznego zamknięcia aplikacji.
- [ ] Sprawdzić odtworzenie w czystym katalogu roboczym: migracje, import, scoring, backend, frontend, aktualizacja i odtworzenie po przerwaniu procesu.
- [ ] Domknąć testy integracyjne bazy i API, smoke test interfejsu oraz CI dla Rust/WASM i kompilacji LaTeX. Testy zdalnych źródeł oddzielić od deterministycznych testów fixture'ów.
- [ ] Usunąć zastąpione skrypty Python/Bash i ich zależności po sprawdzeniu zastępników Rust. Uporządkować puste katalogi po dawnej analityce i nieaktualne instrukcje.
- [ ] Uzupełnić w `thesis/main.tex` problem, źródła, architekturę, metodykę, implementację, testy, eksperyment i ograniczenia. Dodać literaturę uzasadniającą czynniki i ocenę modeli, diagram architektury, model czasu danych i przykładowe wyniki.
- [ ] Sprawdzić zgodność tekstu, wzorów, konfiguracji, danych, tabel i zrzutów ekranu; uzupełnić metadane pracy i deklarację wykorzystania GenAI.
- [ ] Przygotować finalny PDF i scenariusz demonstracji: ranking -> wyjaśnienie spółki -> wpływ makro/COT -> aktualizacja lub jej awaria -> wynik porównania modeli.

Rezultat: działający projekt Rust, odtwarzalne demo, odtwarzalne badanie i finalny tekst pracy.

Warunek ukończenia: inna osoba może odtworzyć demo z instrukcji bez Python/R i bez kont u dostawców. Dostęp do pełnych danych badawczych jest opisany zgodnie z ich warunkami użycia; nie obiecywać publicznej reprodukcji bez kluczy, jeśli wybrane źródło ich wymaga.

## Reguły ograniczania zakresu

Najpierw ograniczać: liczbę wskaźników, liczbę serii makro i rynków COT, szczegóły wyglądu, dodatkowe wykresy, liczbę wariantów analizy wrażliwości i opcjonalny zwrot z dywidendami. Nie rozbudowywać parserów lub architektury ponad konkretne dane potrzebne badaniu.

Zachować jako rdzeń ustalonej wizji: pełny WIG20 w aplikacji, trzy profile, fundamenty z wyceną, trend, aktywne makro, przynajmniej jedno uzasadnione powiązanie COT, automatyczne aktualizacje lokalne, wyjaśnienia wyników, porównanie modeli i Rust w całym kodzie.

Nie ograniczać rygoru przez użycie informacji z przyszłości, ukrywanie braków, pomijanie dawnych uczestników indeksu lub strojenie na końcowym teście. Jeśli dostępność danych wymusi zmianę rdzenia, nazwać tę zmianę przed kontynuacją zależnego zakresu.

Po inżynierce można osobno ocenić sens publicznego produktu, komercyjnych źródeł danych i szerszego uniwersum. Konta, subskrypcje oraz infrastruktura SaaS nie są zależnościami ukończenia tej roadmapy.
