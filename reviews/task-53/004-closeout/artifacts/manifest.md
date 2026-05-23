# Task 53 / 004 — Closeout · Artifact Manifest

Packet path: `reviews/task-53/004-closeout/`
Branch: `task-53` (off `origin/main` `5c0e9e2bd`)
Head SHA: parent of this packet's commit.
Host: Peters-MBP (Apple M5 Pro, 64 GiB, macOS 26.4.1).
PG: 18 on pgrx socket `/Users/peter/.pgrx`, port 28818.

## Surfaces

Closing summary packet. No code change under review (slices 002 +
003 are the canonical source of code review).

## Artifacts

### `suite.json`
Task 53 closeout bench suite config, derived from
`benchmarks/task-50-m5-hnsw-baseline/suite.json` with `name` /
`artifact_dir` / log paths retargeted to this packet. **Same 8-step
shape as baseline**: load + recall + latency + storage at both 10k
and 100k, same prefixes (`ec_real_10k_hnsw`, `ec_real_100k_hnsw`),
same `m: [8, 16]` (10k) and `m: [16]` (100k), same
`ef_construction = 128`, same sweep `[40, 80, 120, 200, 400]`.

### `suite-manifest.json` + `results.jsonl`
`ecaz bench suite run` audit trail (per-step start/end timestamps,
exit codes, log refs) + structured per-step results.

### `corpus-load-{10k,100k}-hnsw.log`
Output of the load steps. These re-build the HNSW indexes through the
Task 53-migrated `source.rs` path (`FlatFloat4Source<'a>::from_datum`,
`DetoastedVarlena::as_typed_slice<f32>`, `AttnumLookup::lookup`).

### `recall-{10k,100k}-hnsw.log`
Recall@10 vs ef sweep. 10k matches baseline exact-equal to four
decimals; 100k deltas inside ci95.

### `latency-{10k,100k}-hnsw.log`
Latency p50/p95/p99 vs ef sweep. 10k slightly faster or equal on
every bucket; **100k 7-13% faster on p50** across all ef buckets —
the wrapper inlining benefit.

### `storage-{10k,100k}-hnsw.log`
Storage per-row + per-index breakdown. Bit-for-bit identical at the
B/row on indexes; total-per-row drift is FSM/VM noise (<1 B at 100k,
0 B at 10k).

### `before-after-summary.md`
Full numeric comparison vs baseline with tolerance assessment. Final
disposition: **Task 53 is a measurable improvement** — no regression
on either corpus, latency consistently faster at 100k.

### `handoff-list.md`
Per task spec §Exit Criterion #4: enumerates SPIRE / IVF / DiskANN
consumer sites that the new wrappers (slice 002) will absorb under
Tasks 55/56/57. Also documents the deferred `DetoastedVarlena<'a>`
lifetime work and the `EcVectorView` shim disposition.

## Validation steps

1. `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`
   (background id `bv4b0vcv7`, release build at HEAD; 10m 19s).
2. `DROP EXTENSION ... CASCADE; CREATE EXTENSION ecaz; DROP TABLE
   ec_real_{10k,100k}_hnsw_{corpus,queries}` to reset bench state
   (operator-authorized for this session).
3. `PGHOST=... PGPORT=28818 ecaz bench suite run --config
   reviews/task-53/004-closeout/artifacts/suite.json` (background id
   `bervpov5d`, full 8-step suite).
4. Compare recall / latency / storage vs
   `benchmarks/task-50-m5-hnsw-baseline/artifacts/` per prefix + ef.

## §Exit Criteria status (final)

| # | Criterion | Status |
| - | --- | --- |
| 1 | Four typed wrappers in src/am/common/datum.rs | **✓** (slice 002) |
| 2 | `src/am/ec_hnsw/source.rs` ≤ 14 | **✓ (13, slice 003)** |
| 3 | HNSW recall + QPS + per-row storage no regression | **✓** (see before-after-summary.md) |
| 4 | Closing summary + SPIRE/IVF/DiskANN handoff list | **✓** (this packet) |

**All four §Exit Criteria satisfied. Task 53 closes 100%.**

- Timestamp: 2026-05-23.
