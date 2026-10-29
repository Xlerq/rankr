# rankr

Local ranking of GPW companies from the bundled WIG20 map, written in Rust.
The program refreshes its data first, then serves the existing three-family
ranking at `http://127.0.0.1:3000/`.

## Run

Install from the repository root:

```bash
cargo install --path importer --locked
```

Start the program without arguments:

```bash
rankr
```

Data lives in `data/collected`, relative to the working directory. Company codes,
ISINs and Yahoo symbols come from `data/raw/wig20_symbols.csv`, embedded in the
binary during compilation. No API keys or database are needed.

## Data refresh

Each launch checks every mapped company separately:

- GPW / Notoria fundamentals: skip a company with today's `fundamentals.json`;
  otherwise download a new snapshot.
- KNF short positions: skip a company with today's `shorts.json`; otherwise
  download a new snapshot. Both daily checks use the UTC date prefix
  `YYYY-MM-DD` of the observation directory.
- Yahoo price history: download the full daily series once for a company without
  history. On subsequent launches, request only dates after its last stored
  candle, up to the current time. Append finished candles to the existing
  `history.json`, without duplicate dates or another full-series request.
  No new sessions means no files are written.

Fundamentals and shorts keep separate dated snapshots. A company's selected
history file grows in place and is replaced atomically, so a failed write leaves
the old file intact. Existing archives with several histories use the latest
`fetched_at`, as the ranking does. Raw responses for nonempty history updates
are kept in dated observation directories.

A failure for one company is printed to stderr and does not stop the other
companies or the page. Axum starts only after the entire refresh finishes.

## Ranking history

After refreshing data, each launch calculates and archives a ranking before
starting the web server. Files are saved under:

```text
data/collected/rankings/YYYY-MM-DD_HH-MM-SS.nnnnnnnnnZ/ranking.json
```

Each file contains the UTC calculation timestamp (`computed_at`), ranking date
(`as_of`), `scoring_version`, and rows with company codes, sectors, all three
family scores and `final_score`. Missing scores stay `null`, while genuine
zeros stay `0`. Empty rankings are not archived.

Earlier snapshots are never overwritten, including repeated launches on the
same day. Reloading the page does not create snapshots. Collection requires
running the program; there is no background schedule or historical backfill.
Snapshots use the data available locally after that launch's refresh, including
older data if some downloads failed. An archive error is printed to stderr and
does not stop the web server. Keep a backup of `data/collected`, which Git ignores.

When changing scoring formulas, thresholds or weights, increment
`SCORING_VERSION` in `importer/src/ranking.rs` to distinguish stored models.

## Ranking page

Open `http://127.0.0.1:3000/`. Each request calculates the ranking for today's UTC
date using `rank_directory`, with columns:

`code`, `gics_sector`, `fundamental_score`, `technical_score`, `sentiment_score`,
`final_score`.

The scoring formulas are unchanged. The final score is the sum of all three
families with weights 1/1/1; a missing family gives a missing final score. Rows
keep the ranking's order. An empty cell means a missing result, while a genuine
zero is displayed as `0`.

Sentiment compares the latest shorts snapshot with the latest snapshot of the
same ISIN from an earlier UTC day. Two empty position lists give zero; a missing
pair gives a missing score. The first snapshot alone cannot produce sentiment.
