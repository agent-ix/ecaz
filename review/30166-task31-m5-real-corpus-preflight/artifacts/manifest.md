# Task 31 M5 Real Corpus Preflight Artifact Manifest

Head SHA: `f7be209c3a3e21f61901df9454c830639eeb5042`

Packet/topic: `review/30166-task31-m5-real-corpus-preflight`

Timestamp: `2026-05-03T01:05:31Z`

Machine: Task 31 M5 laptop from packet `30162`, Apple M5 Pro, macOS local PG18
pgrx environment.

Surface: local M5 PG18 inventory preflight. No recall, latency, storage, build,
or corpus load claims are made by this packet.

CLI path: `/Users/peter/.cargo/bin/ecaz`

Database target: `postgres`, socket directory `/Users/peter/.pgrx`, port `28818`.

## Artifacts

### `pg18-ecaz-status.log`

- Lane: Task 31 M5 real-corpus preflight.
- Fixture: PG18 status only; no corpus surface.
- Storage format: none.
- Rerank mode: none.
- Surface isolation: not applicable.
- Command:
  `/Users/peter/.cargo/bin/ecaz dev sql --pg 18 --db postgres --socket-dir /Users/peter/.pgrx --port 28818 --raw --sql "select version(); select extname, extversion from pg_extension where extname = 'ecaz';" --log-output review/30166-task31-m5-real-corpus-preflight/artifacts/pg18-ecaz-status.log`
- Key result lines:
  - `PostgreSQL 18.3 (Homebrew) on aarch64-apple-darwin25.2.0`
  - `ecaz | 0.1.1`

### `corpus-list.log`

- Lane: Task 31 M5 real-corpus preflight.
- Fixture: loaded-corpus inventory.
- Storage format: existing listed indexes only.
- Rerank mode: existing listed indexes only.
- Surface isolation: inventory command; existing smoke prefix is one synthetic
  corpus table plus indexes from packet `30163`.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30166-task31-m5-real-corpus-preflight/artifacts/corpus-list.log corpus list`
- Key result lines:
  - `task31_m5_smoke_pqg8`
  - `10000`
  - `btree, ec_ivf`
  - `ec_ivf`
- Interpretation: no real DBPedia 10k, 25k, 100k, or 990k Task 31 prefixes
  are currently loaded in `postgres`.
