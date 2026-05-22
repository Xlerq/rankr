OPTION IMPORT;

CREATE instrument:kgh SET
  symbol = "KGH",
  name = "KGHM Polska Miedź SA",
  isin = "PLKGHM000017",
  type = "stock",
  exchange = "GPW",
  currency = "PLN",
  sector = "Mining",
  stooq_symbol = "kgh",
  gpw_code = "KGHM",
  gpwbenchmark_name = "KGHM",
  is_active = true;

CREATE instrument:pko SET
  symbol = "PKO",
  name = "PKO Bank Polski SA",
  isin = "PLPKO0000016",
  type = "stock",
  exchange = "GPW",
  currency = "PLN",
  sector = "Banking",
  stooq_symbol = "pko",
  gpw_code = "PKOBP",
  gpwbenchmark_name = "PKOBP",
  is_active = true;

CREATE instrument:wig20 SET
  symbol = "WIG20",
  name = "WIG20",
  isin = "PL9999999987",
  type = "index",
  exchange = "GPW",
  currency = "PLN",
  sector = NONE,
  stooq_symbol = "wig20",
  gpw_code = "WIG20",
  gpwbenchmark_name = "WIG20",
  is_active = true;

CREATE index_membership:wig20_kgh_2026_05_22 SET
  index = instrument:wig20,
  instrument = instrument:kgh,
  as_of = d"2026-05-22T00:00:00Z",
  index_weight = 8.99,
  source = "gpwbenchmark";

CREATE index_membership:wig20_pko_2026_05_22 SET
  index = instrument:wig20,
  instrument = instrument:pko,
  as_of = d"2026-05-22T00:00:00Z",
  index_weight = 15.38,
  source = "gpwbenchmark";

CREATE price_daily:kgh_2024_01_02 SET
  instrument = instrument:kgh,
  date = d"2024-01-02T00:00:00Z",
  open = 122.00,
  high = 123.00,
  low = 120.00,
  close = 121.50,
  volume = 1000000,
  source = "stooq";

CREATE price_daily:kgh_2024_01_03 SET
  instrument = instrument:kgh,
  date = d"2024-01-03T00:00:00Z",
  open = 121.50,
  high = 124.00,
  low = 121.00,
  close = 123.20,
  volume = 1100000,
  source = "stooq";

CREATE price_daily:wig20_2024_01_02 SET
  instrument = instrument:wig20,
  date = d"2024-01-02T00:00:00Z",
  open = 2300.00,
  high = 2320.00,
  low = 2290.00,
  close = 2310.00,
  volume = NONE,
  source = "stooq";

CREATE instrument:bit11 SET
  symbol = "11B",
  name = "11 bit studios SA",
  isin = "PL11BTS00015",
  type = "stock",
  exchange = "GPW",
  currency = "PLN",
  sector = "Gaming",
  stooq_symbol = "11b",
  gpw_code = "11BIT",
  gpwbenchmark_name = NONE,
  is_active = true;

CREATE fundamental_snapshot:bit11_2025_iv_gpw_notoria SET
  instrument = instrument:bit11,
  report_period = "I-IV kw. 2025",
  report_year = 2025,
  fetched_at = d"2026-05-22T12:32:28Z",
  unit = "tys_pln",
  source = "gpw_notoria",
  source_url = "https://www.gpw.pl/ajaxindex.php?action=GPWListaSp&code=11BIT&format=html&isin=PL11BTS00015&lang=PL&start=showNotoria",

  revenue = 141034.32,
  operating_profit = 21893.18,
  net_income = 6942.67,
  depreciation_amortization = 50224.23,

  assets = 268804.56,
  current_assets = 92773.67,
  equity = 236421.69,
  long_term_liabilities = 3291.49,
  short_term_liabilities = 29091.38,

  operating_cash_flow = 62525.92,
  investing_cash_flow = -82069.54,
  capex = -37993.69,
  financing_cash_flow = -1538.85,
  net_cash_flow = -21082.46,

  ebitda = 72117.40,
  roe = 0.03,
  roa = 0.03,
  current_ratio = 3.19,
  quick_ratio = 3.19,
  debt_service_ratio = 144.81,

  parse_notes = [];

CREATE macro_observation:usdpln_2024_01_02 SET
  date = d"2024-01-02T00:00:00Z",
  series = "USDPLN",
  value = 3.9432,
  unit = "pln_per_usd",
  source = "nbp",
  source_url = "https://api.nbp.pl/api/exchangerates/rates/a/usd/2024-01-01/2024-01-31/?format=json";

CREATE macro_observation:eurpln_2024_01_02 SET
  date = d"2024-01-02T00:00:00Z",
  series = "EURPLN",
  value = 4.3434,
  unit = "pln_per_eur",
  source = "nbp",
  source_url = "https://api.nbp.pl/api/exchangerates/rates/a/eur/2024-01-01/2024-01-31/?format=json";

CREATE macro_observation:gold_pln_2024_01_02 SET
  date = d"2024-01-02T00:00:00Z",
  series = "GOLD_PLN",
  value = 262.94,
  unit = "pln_per_gram",
  source = "nbp",
  source_url = "https://api.nbp.pl/api/cenyzlota/2024-01-01/2024-01-31/?format=json";
