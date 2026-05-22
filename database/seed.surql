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
