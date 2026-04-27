# Artifact Manifest

Packet: `review/30036-task28-ivf-anchor50k-nprobe-debug`

Head SHA: `15a911b593b123ed0b95e148f9c93f914e973819`

Local machine:

- OS: WSL2 Linux `6.6.87.2-microsoft-standard-WSL2`
- CPU: Intel Core i9-10900K, 20 logical CPUs
- Memory: 62 GiB total, 48 GiB available at capture
- PostgreSQL: 18.3, x86_64, gcc 11.4.0
- Storage/cache state: normal local scratch cluster; cache not explicitly
  dropped, so latency results are warm/local smoke numbers only.

## Artifacts

### `pg18-anchor-dimension-check.log`

- command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql 'select ... dimension check ...' --raw --log-output review/30036-task28-ivf-anchor50k-nprobe-debug/artifacts/pg18-anchor-dimension-check.log`
- timestamp: 2026-04-26 local
- purpose: distinguish the old 64-dimensional 10k fixture from the
  1536-dimensional DBPedia anchor.
- key result: anchor corpus/query dim `1536`; old 10k dim `64`.

### `pg18-ivf-anchor50k-n128-nprobe-debug.sql`

- command artifact: attempted 50k x 1536 IVF run.
- lane / fixture / storage / rerank: IVF, DBPedia anchor 50k subset,
  `turboquant`, `rerank = off`.
- isolated surface: copied packet table with one IVF index.

### `pg18-ivf-anchor50k-n128-nprobe-debug.log`

- command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30036-task28-ivf-anchor50k-nprobe-debug/artifacts/pg18-ivf-anchor50k-n128-nprobe-debug.sql --raw --log-output review/30036-task28-ivf-anchor50k-nprobe-debug/artifacts/pg18-ivf-anchor50k-n128-nprobe-debug.log`
- timestamp: 2026-04-26 local
- result: aborted after backend termination; not a benchmark result.
- key partial result: copied `50000` rows with source dimension `1536`.

### `pg18-active-while-50k.log`

- command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql 'select ... from pg_stat_activity ...' --raw --log-output review/30036-task28-ivf-anchor50k-nprobe-debug/artifacts/pg18-active-while-50k.log`
- timestamp: 2026-04-26 local
- key result: `CREATE INDEX task28_ivf_anchor50k_n128_idx` active after
  `00:06:07`.

### `pg18-cancel-heavy-50k-build.log`

- command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql 'select pg_cancel_backend(...)' --raw --log-output review/30036-task28-ivf-anchor50k-nprobe-debug/artifacts/pg18-cancel-heavy-50k-build.log`
- timestamp: 2026-04-26 local
- key result: `pg_cancel_backend = t`.

### `pg18-active-after-cancel.log`

- command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql 'select ... from pg_stat_activity ...' --raw --log-output review/30036-task28-ivf-anchor50k-nprobe-debug/artifacts/pg18-active-after-cancel.log`
- timestamp: 2026-04-26 local
- key result: build still active after `00:10:16`.

### `pg18-terminate-heavy-50k-build.log`

- command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql 'select pg_terminate_backend(...)' --raw --log-output review/30036-task28-ivf-anchor50k-nprobe-debug/artifacts/pg18-terminate-heavy-50k-build.log`
- timestamp: 2026-04-26 local
- key result: `pg_terminate_backend = t`.

### `pg18-active-after-terminate.log`

- command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql 'select ... from pg_stat_activity ...' --raw --log-output review/30036-task28-ivf-anchor50k-nprobe-debug/artifacts/pg18-active-after-terminate.log`
- timestamp: 2026-04-26 local
- key result: backend still active before OS-level kill.

### `pg18-active-anchor10k1536.log`

- command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql 'select ... from pg_stat_activity ...' --raw --log-output review/30036-task28-ivf-anchor50k-nprobe-debug/artifacts/pg18-active-anchor10k1536.log`
- timestamp: 2026-04-26 local
- key result: reduced run had passed build and was active in exact-top10
  materialization.

### `pg18-ivf-anchor10k1536-n32-nprobe-debug.sql`

- command artifact: successful 10k x 1536 nprobe debug run.
- lane / fixture / storage / rerank: IVF, DBPedia anchor 10k subset,
  `turboquant`, `rerank = off`.
- isolated surface: copied packet table with one IVF index.

### `pg18-ivf-anchor10k1536-n32-nprobe-debug.log`

- command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30036-task28-ivf-anchor50k-nprobe-debug/artifacts/pg18-ivf-anchor10k1536-n32-nprobe-debug.sql --raw --log-output review/30036-task28-ivf-anchor50k-nprobe-debug/artifacts/pg18-ivf-anchor10k1536-n32-nprobe-debug.log`
- timestamp: 2026-04-26 local
- key result lines cited by `request.md`:
  - build time: `Time: 24934.108 ms (00:24.934)`
  - index size: `9379840`
  - selected lists / candidates:
    - nprobe 1: selected `1`, candidates `223`, execution `14.888 ms`
    - nprobe 4: selected `4`, candidates `1137`, execution `28.572 ms`
    - nprobe 16: selected `16`, candidates `6016`, execution `108.394 ms`
    - nprobe 32: selected `32`, candidates `10000`, execution `171.477 ms`
  - recall@10:
    - nprobe 1: `0.4400`
    - nprobe 4: `0.6700`
    - nprobe 16: `0.8750`
    - nprobe 32: `0.9200`
