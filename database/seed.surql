OPTION IMPORT;

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
  is_active = true,
  created_at = d"2026-05-22T00:00:00Z",
  updated_at = d"2026-05-22T00:00:00Z";

CREATE instrument:kgh SET
  symbol = "KGH",
  name = "KGHM Polska Miedz SA",
  isin = "PLKGHM000017",
  type = "stock",
  exchange = "GPW",
  currency = "PLN",
  sector = "Mining",
  stooq_symbol = "kgh",
  gpw_code = "KGHM",
  gpwbenchmark_name = "KGHM",
  is_active = true,
  created_at = d"2026-05-22T00:00:00Z",
  updated_at = d"2026-05-22T00:00:00Z";

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
  is_active = true,
  created_at = d"2026-05-22T00:00:00Z",
  updated_at = d"2026-05-22T00:00:00Z";

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
  is_active = true,
  created_at = d"2026-05-22T00:00:00Z",
  updated_at = d"2026-05-22T00:00:00Z";

CREATE index_membership:wig20_kgh_2026_05_21 SET
  index = instrument:wig20,
  instrument = instrument:kgh,
  as_of = d"2026-05-21T00:00:00Z",
  index_weight = 8.999,
  source = "gpwbenchmark";

CREATE index_membership:wig20_pko_2026_05_21 SET
  index = instrument:wig20,
  instrument = instrument:pko,
  as_of = d"2026-05-21T00:00:00Z",
  index_weight = 15.382,
  source = "gpwbenchmark";

CREATE price_daily:kgh_2024_01_02_stooq SET
  instrument = instrument:kgh,
  date = d"2024-01-02T00:00:00Z",
  open = 121.871,
  high = 122.614,
  low = 117.812,
  close = 118.951,
  volume = 475373.24396983,
  source = "stooq";

CREATE price_daily:kgh_2024_01_03_stooq SET
  instrument = instrument:kgh,
  date = d"2024-01-03T00:00:00Z",
  open = 118.703,
  high = 118.951,
  low = 114.644,
  close = 116.03,
  volume = 649641.44760554,
  source = "stooq";

CREATE price_daily:wig20_2024_01_02_stooq SET
  instrument = instrument:wig20,
  date = d"2024-01-02T00:00:00Z",
  open = 2344.10,
  high = 2350.52,
  low = 2288.30,
  close = 2303.41,
  volume = 13920658,
  source = "stooq";

CREATE fundamental_snapshot:bit11_2025_iv_gpw_notoria SET
  instrument = instrument:bit11,
  report_period = "I-IV kw. 2025",
  report_year = 2025,
  fetched_at = d"2026-05-22T12:32:28Z",
  unit = "tys. PLN",
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
  roe = 0.029,
  roa = 0.026,
  current_ratio = 3.19,
  quick_ratio = 3.19,
  debt_service_ratio = 144.81,
  raw_hash = "sample_11bit_gpw_notoria_2026_05_22",
  parse_notes = ["Seed uses a small GPW / Notoria sample, not a full fundamentals history."];

CREATE macro_observation:usdpln_2024_01_02_nbp SET
  date = d"2024-01-02T00:00:00Z",
  series = "USDPLN",
  value = 3.9432,
  unit = "PLN per USD",
  source = "nbp",
  source_url = "https://api.nbp.pl/api/exchangerates/rates/a/usd/2024-01-01/2024-01-31/?format=json";

CREATE macro_observation:eurpln_2024_01_02_nbp SET
  date = d"2024-01-02T00:00:00Z",
  series = "EURPLN",
  value = 4.3434,
  unit = "PLN per EUR",
  source = "nbp",
  source_url = "https://api.nbp.pl/api/exchangerates/rates/a/eur/2024-01-01/2024-01-31/?format=json";

CREATE macro_observation:gold_pln_2024_01_02_nbp SET
  date = d"2024-01-02T00:00:00Z",
  series = "GOLD_PLN",
  value = 262.94,
  unit = "PLN per gram",
  source = "nbp",
  source_url = "https://api.nbp.pl/api/cenyzlota/2024-01-01/2024-01-31/?format=json";

CREATE score_config:default_fundamental_v1 SET
  name = "default_fundamental_v1",
  description = "Default fundamental-first scoring configuration. Fundamentals dominate the score, trend is only a small filter, and macro context is disabled for MVP.",
  scope = "global",
  sector = NONE,
  profitability_weight = 0.35,
  financial_strength_weight = 0.25,
  cashflow_quality_weight = 0.20,
  efficiency_weight = 0.10,
  trend_weight = 0.10,
  macro_context_weight = 0.00,
  is_default = true,
  is_active = true,
  created_at = d"2026-05-22T00:00:00Z",
  updated_at = d"2026-05-22T00:00:00Z";

CREATE score_result:bit11_2026_05_22_default_fundamental_v1 SET
  instrument = instrument:bit11,
  config = score_config:default_fundamental_v1,
  score_date = d"2026-05-22T00:00:00Z",
  fundamental_snapshot = fundamental_snapshot:bit11_2025_iv_gpw_notoria,
  profitability_score = 62.00,
  financial_strength_score = 90.00,
  cashflow_quality_score = 85.00,
  efficiency_score = 55.00,
  trend_score = 50.00,
  macro_context_score = 50.00,
  final_score = 71.70,
  label = "good",
  data_quality_score = 90.00,
  explanation = "Seed example for fundamental-first scoring based on GPW / Notoria fundamentals and default scoring config.";

CREATE data_source_log:stooq_kgh_sample_2024_01 SET
  source = "stooq",
  operation = "fetch_price_sample",
  status = "success",
  instrument = instrument:kgh,
  target_symbol = "kgh",
  started_at = d"2026-05-22T12:00:00Z",
  finished_at = d"2026-05-22T12:00:02Z",
  rows_count = 22,
  date_from = d"2024-01-01T00:00:00Z",
  date_to = d"2024-01-31T00:00:00Z",
  raw_file_path = "data/raw/kgh_stooq_sample.csv",
  error_message = NONE,
  notes = "Phase 1 Stooq KGH OHLCV sample.";

CREATE data_source_log:stooq_wig20_sample_2024_01 SET
  source = "stooq",
  operation = "fetch_price_sample",
  status = "success",
  instrument = instrument:wig20,
  target_symbol = "wig20",
  started_at = d"2026-05-22T12:00:00Z",
  finished_at = d"2026-05-22T12:00:02Z",
  rows_count = 22,
  date_from = d"2024-01-01T00:00:00Z",
  date_to = d"2024-01-31T00:00:00Z",
  raw_file_path = "data/raw/wig20_stooq_sample.csv",
  error_message = NONE,
  notes = "Phase 1 Stooq WIG20 OHLCV sample.";

CREATE data_source_log:gpwbenchmark_wig20_sample_2026_05_22 SET
  source = "gpwbenchmark",
  operation = "fetch_index_membership_sample",
  status = "success",
  instrument = instrument:wig20,
  target_symbol = "WIG20",
  started_at = d"2026-05-22T12:10:00Z",
  finished_at = d"2026-05-22T12:10:03Z",
  rows_count = 20,
  date_from = NONE,
  date_to = NONE,
  raw_file_path = "data/raw/wig20_portfolio_gpwbenchmark_sample.json",
  error_message = NONE,
  notes = "Phase 1 GPW Benchmark WIG20 portfolio sample.";

CREATE data_source_log:gpw_notoria_11bit_sample_2026_05_22 SET
  source = "gpw_notoria",
  operation = "fetch_fundamentals_sample",
  status = "success",
  instrument = instrument:bit11,
  target_symbol = "11BIT",
  started_at = d"2026-05-22T12:32:00Z",
  finished_at = d"2026-05-22T12:32:28Z",
  rows_count = 1,
  date_from = NONE,
  date_to = NONE,
  raw_file_path = "data/raw/11bit_gpw_notoria_sample.json",
  error_message = NONE,
  notes = "Phase 1 GPW / Notoria fundamentals sample.";

CREATE data_source_log:nbp_macro_sample_2024_01 SET
  source = "nbp",
  operation = "fetch_macro_sample",
  status = "success",
  instrument = NONE,
  target_symbol = "USDPLN,EURPLN,GOLD_PLN",
  started_at = d"2026-05-22T12:20:00Z",
  finished_at = d"2026-05-22T12:20:04Z",
  rows_count = 66,
  date_from = d"2024-01-01T00:00:00Z",
  date_to = d"2024-01-31T00:00:00Z",
  raw_file_path = "data/raw/nbp_*_sample.json",
  error_message = NONE,
  notes = "Phase 1 NBP macro samples: USDPLN, EURPLN and gold.";
