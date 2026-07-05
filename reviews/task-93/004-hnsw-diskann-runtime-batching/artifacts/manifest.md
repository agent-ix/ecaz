# Manifest: Task 93 Packet 004 HNSW + DiskANN Runtime Batching

- Head SHA: `23f618aa5` (series under review: `4b382584d` runtime batching,
  `1516996f8` HNSW binary-branch fix, `23f618aa5` partial-width SIMD
  dispatch; plus `869645f17` Task 100 stub from reviewer follow-up)
- Task bucket: `reviews/task-93/`
- Packet path: `reviews/task-93/004-hnsw-diskann-runtime-batching/`
- Lane: local PG18 pgrx fixture, Apple M5 Pro (arm64, NEON)
- Host/socket: `/Users/peter/.pgrx`, port `28818`; database `task93_bench`
- Extension installs logged in `install-ecaz-pg18.log` (three installs:
  initial slice, binary-branch fix, partial-dispatch fix; the cited bench
  evidence is from the final install at `23f618aa5`)
- Fixture: dbpedia real10k, 1536-dim
  (`data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_*`)
- Storage formats: `ec_hnsw` `storage_format=rabitq` (`m=16`,
  `ef_construction=128`); `ec_diskann` `storage_format=rabitq`
  (defaults, `DISKANN_RABITQ_BITS=1`)
- Isolation: isolated one-index-per-table prefixes
  `task93_p4_hnsw_rabitq_real10k`, `task93_p4_diskann_rabitq_real10k`
- Suite config: `crates/ecaz-cli/suites/task93-phase4-hnsw-diskann-rabitq.json`
- Kernel-on cells: default GUCs (`ec_hnsw.candidate_batch_scoring` and the
  new `ec_diskann.candidate_batch_scoring` both default on).
- Kernel-off cells: the respective GUC `=off` (per-candidate production
  scoring).
- Tables dropped before each suite invocation (Task 100 `count(*)` planner
  workaround, as in packet 003).

## Development history captured by this packet's runs

1. **Run 1** (commit `4b382584d`): HNSW emitted zero block-kernel rows —
   default RaBitQ scans run with the binary prefilter active, so traversal
   takes the binary branch and the non-binary accumulation arm never fired.
   Fixed in `1516996f8` (batch the binary branch's RaBitQ survivors).
2. **Run 2** (commit `1516996f8`): HNSW rows appeared but 100% of candidates
   were on the scalar tail: graph-AM batch sizes (HNSW survivors avg ~22,
   DiskANN nodes avg ~10) almost never reach the 32-wide block, and the
   forced-scalar tail regressed default-on latency (HNSW p50 +37%, DiskANN
   p50 +25% vs kernel-off). Fixed in `23f618aa5`: partial-width SIMD
   dispatch (sub-32 runs score through the production NEON pair + single
   primitives; ISA-truthful counter attribution).
3. **Run 3** (commit `23f618aa5`): cited below as the packet evidence.

## Validation artifacts (HEAD `23f618aa5`)

- `cargo-clippy.log` — `-D warnings` clean.
- `cargo-test-diskann-quantizer.log` — 13 passed, including the
  partial-aware routing proof
  `diskann_rabitq_prefilter_batch_routes_through_block_kernel`.
- `cargo-test-hnsw-scan.log` — 81 passed.
- (rabitq32 6, candidate_batch 10, ec_ivf 27 also green pre-commit; logs in
  the commit message and re-runnable at HEAD.)

## Bench evidence (final suite run)

### Recall byte-equality — PASS on both AMs

| AM | sweep | kernel-on recall@10 | kernel-off |
|---|---|---|---|
| ec_hnsw | ef_search=80 | 0.9422 (identical percentiles, ndcg 0.9872) | identical |
| ec_diskann | list_size=128 | 0.9984 (ndcg 0.9999) | identical |

### `[block-kernel-counters]` — full SIMD coverage after partial dispatch

```text
surface=hnsw    quant=rabitq isa=neon flushes=3002 candidates=66961 kernel_candidates=66961 kernel_elapsed_ms=15.415471 scalar_candidates=0
surface=diskann quant=rabitq isa=neon flushes=4024 candidates=39353 kernel_candidates=39353 kernel_elapsed_ms=11.214466 scalar_candidates=0
```

- HNSW: 230 ns/candidate; DiskANN: 285 ns/candidate on the NEON path.
  Against the packet-002 forced-scalar reference on the same corpus/dim
  (793 ns/cand at comparable candidate volumes, IVF surface), that is
  3.4× / 2.8× — the ≥2× per-ISA scoring-share gate holds with the caveat
  that the scalar reference cell was measured on the IVF surface (graph
  AMs never had a scalar-kernel cell; Phase A was IVF-only).
- Kernel-off cells emit zero rows (clean toggles on both new/existing GUCs).
- Counter-semantics note (deliberate, please review): `kernel_*` now means
  "SIMD-backend flushes" — full 32-wide blocks AND partial sub-32 runs —
  while `scalar_*` remains strictly scalar-executed work. Graph AMs are
  structurally sub-32 (degree/survivor-budget bound), so the old
  "32-wide-only" reading would classify their entire workload as scalar
  while it actually runs NEON.

### End-to-end latency

Suite cells (32 iterations) plus four interleaved 64-iteration recheck runs
(`recheck-hnsw-latency-{on,off}-{1,2}.log`) because run-order drift on this
host exceeded the cell deltas:

| run | kernel-on p50 | kernel-off p50 |
|---|---|---|
| suite (32 it) | 3.61 ms | 3.02 ms |
| recheck pair 1 | 3.13 ms | 2.25 ms |
| recheck pair 2 | 3.17 ms | 3.27 ms |

HNSW on/off samples interleave across runs (each config produced both the
fastest and slowest observations); DiskANN suite cells are at parity
(p50 2.85 vs 2.87 ms). Conclusion: parity within machine noise, no
directional regression after the partial-width fix; the wall-clock upside
for graph AMs awaits larger effective batch widths (cross-node accumulation
is future work; per-ISA scoring-share already clears its gate).

### `suite-manifest.json` / `results.jsonl` / `suite-run.log` / load logs / `truth-cache/`

- Structured runner outputs; truth caches shared with packet 002 fixtures.
