# Manifest: Task 93 Packet 006 Graph-AM 50k/100k Matrix

- Head SHA: `235412816` (no code change in this packet; pure measurement of
  the packet-004/005 state — the extension installed for this run is the
  packet-005 lineage build, install log shared with packet 004's third
  install)
- Task bucket: `reviews/task-93/`
- Packet path: `reviews/task-93/006-graph-am-50k-100k-matrix/`
- Lane: local PG18 pgrx fixture, Apple M5 Pro (arm64, NEON)
- Host/socket: `/Users/peter/.pgrx`, port `28818`; database `task93_bench`
- Fixtures: dbpedia real50k (`data/task31_m5_dbpedia_staged/ec_hnsw_real_50k_*`)
  and real100k (`data/task60_m5_dbpedia_staged/ec_real_100k_*`), 1536-dim
- Storage formats / reloptions: as packet 004 (HNSW rabitq m=16
  ef_construction=128; DiskANN rabitq defaults)
- Isolation: prefixes `task93_p5_{hnsw,diskann}_rabitq_real{50k,100k}`
- Suite config: `crates/ecaz-cli/suites/task93-phase5-graph-50k-100k.json`
- Cells: kernel-on (default GUCs) vs kernel-off
  (`ec_hnsw.candidate_batch_scoring=off` / `ec_diskann.candidate_batch_scoring=off`)

## Recall byte-equality — PASS at every cell

| AM | corpus | kernel-on recall@10 | kernel-off |
|---|---|---|---|
| ec_hnsw | real50k | 0.8812 | identical |
| ec_hnsw | real100k | 0.8906 | identical |
| ec_diskann | real50k | 0.9917 | identical |
| ec_diskann | real100k | 0.9781 | identical |

## `[block-kernel-counters]` — full NEON coverage, clean toggles

| AM | corpus | candidates (all on `isa=neon`) | kernel ns/cand |
|---|---|---|---|
| hnsw | 50k | 65,098 | 171 |
| hnsw | 100k | 72,703 | 137 |
| diskann | 50k | 63,030 | 267 |
| diskann | 100k | 73,322 | 236 |

Kernel-off cells emit zero block-kernel rows. Against the packet-002
forced-scalar reference (793/514/364 ns/cand on the same corpora, IVF
surface), every cell clears the ≥2× per-ISA scoring-share gate
(2.9×–5.8×), with the same cross-surface caveat noted in packet 004.

## End-to-end latency

Suite cells (32 iterations) plus interleaved 64-iteration rechecks for the
100k cells (`recheck-*.log`), because run-order drift on this host exceeds
cell deltas (established in packet 004):

| cell | suite on/off p50 | recheck on p50 | recheck off p50 |
|---|---|---|---|
| hnsw 50k | 5.55 / 5.53 ms | — | — |
| hnsw 100k | 5.91 / 5.23 ms | 3.62, 3.66 ms | 3.77, 4.24 ms |
| diskann 50k | 5.16 / 5.34 ms | — | — |
| diskann 100k | 6.82 / 5.82 ms | 3.13 ms | 3.94 ms |

The suite-run 100k deltas invert under interleaved rechecks: kernel-on is
at parity (hnsw) or faster (diskann). No directional regression at any
cell.

## Measured (AM × corpus × ISA) matrix to date (M5 lane)

| AM | corpus | scalar kernel | NEON kernel | recall gate |
|---|---|---|---|---|
| ivf | real10k | 793/515 ns/cand (packet 002) | 223/191 ns/cand (packet 003) | byte-equal |
| ivf | real100k | 364 ns/cand (packet 002) | 126 ns/cand (packet 003) | byte-equal |
| hnsw | real10k | — (graph AMs entered at NEON phase) | 230 ns/cand (packet 004) | byte-equal |
| hnsw | real50k/100k | — | 171 / 137 ns/cand (this packet) | byte-equal |
| diskann | real10k | — | 285 ns/cand (packet 004) | byte-equal |
| diskann | real50k/100k | — | 267 / 236 ns/cand (this packet) | byte-equal |

SVE (Graviton) and AVX2 (Intel desktop) columns remain deferred per packet
005's lane plan.

## Artifacts

Suite outputs (`suite-manifest.json`, `results.jsonl`, `suite-run.log`),
per-cell load/recall/latency logs, 100k recheck logs, shared truth caches.
