# Task 54/005 Closeout — Bench Artifact Manifest

Post-Task-54 HNSW bench window. Validates §Exit Criterion #3 — no
regression vs the post-Task-50 M5 baseline at
`benchmarks/task-50-m5-hnsw-baseline/manifest.md` after the P3 page/WAL/buffer wrapper
migration in HNSW `build.rs` (packet 003) + `vacuum.rs` (packet 004).

## Head and host

| Field | Value |
| --- | --- |
| HEAD SHA | `cd7fe728b` (pre-005 reviewer commit) — bench artifacts captured against the extension built from this HEAD via `cargo pgrx install --release`; the slice-005 packet commit lands after the artifacts |
| Task | 54 |
| Packet | 005 closeout |
| Captured | 2026-05-23 (America/Los_Angeles) |
| Host | Peters-MBP (Apple Silicon M5 Pro, 64 GiB) |
| OS | macOS 26.4.1 (Darwin 25.4.0 arm64) |
| PostgreSQL | 18 (pgrx local install, socket `/Users/peter/.pgrx`, port 28818) |
| Extension build | `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config` (see `pgrx-install.log` upstream) |

## Scope

HNSW only (Task 54 §Scope HNSW-only lock). Corpora `ec_real_10k`
(10k / 200 queries) and `ec_real_100k` (100k / 1000) from
`fixtures/m5_diskann_real{10k,100k}/`.

Sweep:

- HNSW build: `m ∈ {8, 16}` at 10k, `m = 16` at 100k, `ef_construction = 128`.
- HNSW scan: `ef_search ∈ {40, 80, 120, 200, 400}`, `k = 10`.

Same shape as Task 50/449's M5 baseline at
`benchmarks/task-50-m5-hnsw-baseline/`.

## Run command

```sh
/Users/peter/.cargo/bin/ecaz \
  --host /Users/peter/.pgrx --port 28818 --database tqvector_bench \
  bench suite run \
  --config reviews/task-54/005-closeout/suite.json \
  --log-file reviews/task-54/005-closeout/artifacts/suite-run.log
```

Suite name `task-54-005-closeout`, derived from the baseline's
`suite.json` with `artifact_dir` redirected to this packet.

## Artifacts

| Step | Log |
| --- | --- |
| load 10k HNSW | `corpus-load-ec_real_10k-hnsw.log` |
| recall 10k HNSW | `recall-ec_real_10k-hnsw.log` |
| latency 10k HNSW | `latency-ec_real_10k-hnsw.log` |
| storage 10k HNSW | `storage-ec_real_10k-hnsw.log` |
| load 100k HNSW | `corpus-load-ec_real_100k-hnsw.log` |
| recall 100k HNSW | `recall-ec_real_100k-hnsw.log` |
| latency 100k HNSW | `latency-ec_real_100k-hnsw.log` |
| storage 100k HNSW | `storage-ec_real_100k-hnsw.log` |
| Suite | `results.jsonl`, `suite-manifest.json`, `suite-run.log` |
| Summary | `before-after-summary.md` |

## Key result lines (cited from `request.md`)

(Filled by run.)

## Notes

The build/vacuum paths now route through the P3 wrappers:
`LockedBufferGuard::read_main{,_locked}_handle`, `WalTxnScope::start_handle`,
and `RegisteredBufferPage::{init,add_item}`. WAL semantics
(`GenericXLogStart` / `RegisterBuffer` / `Finish` / `Abort`) are
unchanged; the wrappers are call-site moves only.
