# rankr — punktacja i przepływ danych

Ranking WIG20 powstaje z sumy trzech rodzin punktów. Wynik jest umownym sygnałem
względnego potencjału wzrostu, nie prognozą zwrotu w procentach.

## Źródła danych

| Rodzina | Źródło | Dane lokalne |
| --- | --- | --- |
| Fundamental | GPW / Notoria | `fundamentals.json`: jeden bieżący raport finansowy |
| Technical | Yahoo Finance, bez klucza API | `history.json`: wieloletnie świece dzienne |
| Sentiment | Publiczny rejestr krótkiej sprzedaży KNF | `shorts.json`: ujawnione pozycje według ISIN spółki |

## Przepływ danych

`rankr` → lista spółek z `data/raw/wig20_symbols.csv` → pobieranie → surowe
odpowiedzi `raw.json` i parsowanie → pliki w `data/collected` → obliczenia → tabela HTML.

Fundamenty i shorty są pobierane najwyżej raz dziennie na spółkę, według dnia UTC.
Rejestr KNF jest pobierany wspólnie i filtrowany lokalnie. Historia cen jest
pobierana raz w całości, potem uzupełniana o brakujące świece bez duplikatów.
Błąd jednej spółki nie zatrzymuje reszty.

Po odświeżaniu startuje `http://127.0.0.1:3000/`. Wejście na stronę liczy ranking
na dziś UTC z lokalnych danych, bez sieci. Fundamenty i historię wybiera według
najnowszego `fetched_at`.

## Technical: sezonowość + trend

`technical_score = seasonality_points + trend_points`, zakres **od −2 do +2**.
Każdy składnik daje −1, 0 albo +1. Liczą się tylko świece sprzed daty rankingu.

- **Sezonowość:** średnia zwrotów z tego samego miesiąca kalendarzowego
  z poprzednich lat. Zwrot = cena końca miesiąca / cena końca poprzedniego
  miesiąca − 1. Ceny pochodzą z ostatnich sesji; używane jest `adjusted_close`,
  z zastąpieniem przez `close`, gdy go brakuje.
  **Średnia > 2% → +1, < −2% → −1, inaczej 0.**
- **Trend:** ostatnie `close` względem SMA200 — średniej z 200 dziennych `close`.
  **Odchylenie < 3% → 0; od 3%: powyżej średniej → +1, poniżej → −1.**
  Mniej niż 200 sesji → trend **0**.

Brak choć jednego historycznego miesiąca do obliczenia sezonowości lub ostatnia
świeca starsza niż 10 dni kalendarzowych → `technical_score = null`.

## Fundamental: wskaźniki z jednego raportu

Kwoty uwzględniają `unit_multiplier`; obliczenia używają `Decimal`.
Zysk netto i kapitał własny dotyczą udziałowców jednostki dominującej.

| Wskaźnik | Wzór | +1 | −1 |
| --- | --- | --- | --- |
| `net_margin` | zysk netto / przychody | > 0,02 | < −0,02 |
| `operating_margin` | zysk operacyjny / przychody | > 0,02 | < −0,02 |
| `equity_ratio` | kapitał własny / aktywa | > 0,50 | < 0,30 |
| `debt_to_equity` | (zobowiązania długoterminowe + krótkoterminowe) / kapitał własny | 0 ≤ x < 0,50 | < 0 lub > 1,50 |
| `ocf_to_net_income` | przepływy operacyjne / zysk netto | > 1 | < 0 |

Pozostałe wartości dają **0**. Brak danych lub mianownik 0 → **null**.
Oba składniki zobowiązań muszą być dostępne. Zysk netto ≤ 0 oznacza
`ocf_to_net_income = null`.

**Financials (GICS 40):** punktowane jest tylko `equity_ratio`:
**powyżej 0,10 → +1, poniżej 0,08 → −1, inaczej 0**.
Pozostałe cztery wskaźniki i punkty są `null`, także dla PZU i KRUK.

`fundamental_score` jest sumą dostępnych punktów; brak wszystkich daje `null`.
Zakres wynosi **od −5 do +5**, a dla Financials **od −1 do +1**.
GICS wybiera szablon, bez porównań między spółkami. Progi są konwencją,
bez kalibracji na WIG20. Brak wyceny i dynamiki rok do roku.

## Sentiment: zmiana ujawnionych pozycji krótkich

Porównywane są sumy procentów pozycji: najnowszy `shorts.json` i najnowszy starszy
snapshot tego samego ISIN **z wcześniejszego dnia UTC**, bez wymogu pełnych 24 godzin.
Na posiadacza zostaje jeden najnowszy wpis, bez sumowania historycznych zgłoszeń.

- Suma wzrosła → **−1**.
- Suma spadła → **+1**.
- Suma nie zmieniła się, także przy dwóch pustych listach → **0**.
- Brak pary snapshotów → **null**.

Zmiana posiadaczy nie wyklucza porównania. Pusta lista oznacza brak publicznych
wpisów, nie optymizm rynku. Ten sygnał nie obejmuje całego sentymentu inwestorów.

## Wynik końcowy

`final_score = fundamental_score + technical_score + sentiment_score`

Wagi: **1 / 1 / 1**. Brak dowolnej rodziny → wynik **null**, czyli pusta komórka;
zero pozostaje zerem. Sortowanie: malejąco, `null` na końcu, remis po kodzie.
Równe wagi nie wyrównują różnych zakresów punktów poszczególnych rodzin.
