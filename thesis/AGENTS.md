# Zasady pisania pracy inżynierskiej

Te instrukcje określają sposób pracy nad tekstem w `thesis/`. Samo zapisanie
instrukcji nie oznacza polecenia przeredagowania pracy. Zakres następnych zmian
wynika z kolejnego zadania użytkownika.

## Bieżący etap redakcji

- Utrzymuj rozwinięte rozdziały 1 i 2: analizę problemu i źródeł danych oraz
  architekturę systemu.
- Pozostałe rozdziały mają na razie być krótkimi szkicami: mało zdań, tylko
  najważniejsze i sprawdzone informacje. Rozwijaj je szerzej dopiero na polecenie
  użytkownika.
- Miejsca wymagające rozwinięcia oznaczaj komentarzami TODO w LaTeX. Roboczy
  charakter tekstu ma wynikać z ograniczonego zakresu, przy zachowaniu poprawności
  języka i zgodności z aktualnym kodem.

## Styl i narracja

- Pisz po polsku, naturalnie i objaśniająco, z zachowaniem precyzji pracy
  inżynierskiej. Przykład docelowego stylu: „Program zapisuje pobrane dane.
  Dzięki temu obliczenia można później powtórzyć bez łączenia się z dostawcą”.
- Przy opisie działania używaj konkretnego podmiotu: „program pobiera”,
  „parser odczytuje”, „system przechowuje”. Przy decyzjach i wykonanych pracach
  stosuj formy bezosobowe: „wybrano”, „przyjęto”, „zaimplementowano”.
- Nie używaj pierwszej osoby liczby pojedynczej ani mnogiej. Unikaj również
  zastępowania jej powtarzanym określeniem „autor pracy”.
- Zachowaj swobodny tok wyjaśnienia, bez potocznych zwrotów, tonu rozmowy
  z czytelnikiem i urzędowego języka. Preferuj „zapisano” zamiast „dokonano
  zapisu”. Usuwaj puste zapowiedzi, ozdobniki i powtórzenia.
- Akapit zwykle powinien mieć 4–6 zdań i rozwijać jedno zagadnienie. Nie jest
  to sztywny limit: nie dopisuj treści wyłącznie dla osiągnięcia tej długości.
  Zdania mają tworzyć spójne wyjaśnienie, a nie serię oderwanych stwierdzeń.

## Czytelnik i poziom wyjaśnień

- Pisz dla osoby rozpoczynającej pierwszy rok inżynierii i analizy danych,
  z wiedzą po zdanej maturze. Nie zakładaj znajomości finansów, architektury
  aplikacji, Rusta, baz danych ani statystyki na poziomie studiów.
- Wyjaśniaj potrzebne pojęcia i skróty przy pierwszym użyciu, zanim oprzesz na
  nich dalszy opis. Podawaj ich znaczenie oraz rolę w projekcie. Nie zastępuj
  jednego nieznanego terminu kilkoma innymi bez objaśnienia.
- W razie potrzeby przejdź od prostego przykładu do reguły ogólnej. Przy wzorach
  wyjaśnij symbole, jednostki, sposób odczytania wyniku i jego zastosowanie.
- Zapewnij podstawy potrzebne do zrozumienia pracy, ale nie twórz podręcznika
  niezwiązanego z projektem. Raz wyjaśnione pojęcia wykorzystuj konsekwentnie;
  w dalszych częściach wystarczy krótkie przypomnienie lub odwołanie.

## Układ i szczegółowość

- Podstawy finansowe przedstawiaj w osobnej części teoretycznej. Technologie
  objaśniaj przy architekturze i implementacji, kiedy stają się potrzebne.
- W opisie rozwiązania skupiaj się na decyzjach projektowych: problemie,
  wymaganiu, przyjętym rozwiązaniu i jego konsekwencjach. Uzupełniaj ten opis
  krótkim wyjaśnieniem budowy programu oraz współpracy jego części.
- Dla istotnego wyboru porównaj przyjęte rozwiązanie z jedną realną alternatywą.
  Wskaż kryterium wyboru i kompromis. Nie twórz rozbudowanego katalogu narzędzi
  ani sztucznych porównań dla drobnych szczegółów implementacji.
- Używaj nazw plików, struktur, funkcji, pól i komend tylko wtedy, gdy pomagają
  zrozumieć rozwiązanie. Główny tekst nie ma być katalogiem kodu ani instrukcją
  obsługi CLI. Nazwy i identyfikatory z kodu pozostawiaj w oryginalnej postaci.
- Stosuj akapity do wyjaśnień, tabele do porównań, a listy do wymagań i kolejnych
  kroków. Schematy i przykłady dodawaj wtedy, gdy ułatwiają zrozumienie.
- Większość ograniczeń i zastrzeżeń zbieraj w wydzielonej części o ograniczeniach
  i walidacji. Nie powtarzaj tych samych uwag w wielu rozdziałach. Przy opisie
  metody pozostaw jednak warunek lub krótkie odwołanie, jeśli bez niego
  czytelnik błędnie zrozumiałby jej działanie albo znaczenie wyniku.

## Stan projektu i rzetelność

- Pisz od razu w strukturze pracy końcowej, bez zamieniania jej w datowany
  dziennik postępu. Brakujące fragmenty oznaczaj komentarzami `% TODO(thesis):`
  w LaTeX, zamiast rozbudowanymi zapowiedziami w tekście dla czytelnika.
- Strukturę docelową odróżniaj od stanu realizacji: nie opisuj planowanej
  funkcjonalności, pomiarów ani badań jako wykonanych. Opis świadomie
  projektowanego rozwiązania musi jasno wskazywać, że jest projektem.
- Sprawdzaj fakty dotyczące implementacji w kodzie, konfiguracji i dostępnych
  wynikach. Nie dopisuj fikcyjnych uzasadnień decyzji, rezultatów, parametrów
  ani wniosków. Oddzielaj fakty, założenia projektowe i interpretacje.
- Twierdzenia o wydajności i skuteczności rankingu opieraj na rzeczywistych
  pomiarach i odpowiedniej metodzie oceny. Poprawne wykonanie programu nie
  jest dowodem skuteczności inwestycyjnej.

## Źródła

- Samodzielnie wyszukuj i weryfikuj źródła podczas pracy nad tekstem. Nie
  wymagaj osobnego zatwierdzania każdej pozycji bibliograficznej.
- Preferuj publikacje naukowe, oficjalną dokumentację i materiały instytucji
  odpowiedzialnych za opisywane dane lub metody. Dobieraj źródło do konkretnego
  twierdzenia i sprawdzaj, czy faktycznie je potwierdza.
- Cytuj materiały, z którymi rzeczywiście się zapoznano. Sprawdzaj dane
  bibliograficzne, adresy i zgodność cytowania z treścią. Nie wymyślaj źródeł,
  autorów, dat, numerów stron ani identyfikatorów publikacji.
- Jeśli brakuje potwierdzenia, zawęź twierdzenie do tego, co ustalono, albo
  pozostaw komentarz TODO i zgłoś konkretną lukę. Nie maskuj jej cytowaniem
  ogólnego źródła.

## Kolejność pracy przy następnym zadaniu

1. Ustal zakres z bieżącego polecenia i przeczytaj wskazany fragment wraz
   z otoczeniem. Sprawdź miejscowe instrukcje, wymagania formalne i uwagi
   promotora, jeśli są dostępne. Nie rozszerzaj zadania na całą pracę bez potrzeby.
2. Zweryfikuj stan opisywanych elementów w repozytorium. Oddziel elementy
   wykonane, zaprojektowane i brakujące; ustal, które wymagają komentarza TODO.
3. Ułóż tok wyjaśnienia dla początkującego czytelnika: potrzebne pojęcie,
   problem, decyzja i jej powód, sposób działania oraz konsekwencje. Dostosuj
   kolejność do fragmentu, bez narzucania identycznego schematu każdemu akapitowi.
4. Wyszukaj i sprawdź źródła potrzebne do twierdzeń teoretycznych i zewnętrznych.
   Dla uzasadnianej decyzji wybierz jedną rzeczywistą alternatywę do porównania.
5. Napisz lub przeredaguj wyłącznie uzgodniony zakres w stylu opisanym powyżej.
   Zachowaj zgodność terminologii, oznaczeń i odwołań z resztą pracy.
6. Przeczytaj tekst jako student pierwszego roku: wyjaśnij brakujące pojęcia,
   usuń skoki w rozumowaniu, zbędne nazwy z kodu, powtórzenia i nadmiar
   zastrzeżeń. Sprawdź narrację, fakty i przypisanie źródeł.
7. Sprawdź zmiany i poprawność LaTeX w zakresie odpowiednim do edycji. Przy
   zmianach pracy skompiluj dokument, jeśli dostępne są narzędzia. W odpowiedzi
   krótko podaj zakres zmian, wynik weryfikacji i rzeczywiste braki do uzupełnienia.
