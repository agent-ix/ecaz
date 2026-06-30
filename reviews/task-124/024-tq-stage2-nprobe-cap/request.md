# Task 124 Packet 024: TQ stage-2 nprobe cap

## Summary

This packet covers the first post-reopen TQ speed code slice after packet 017.
The slice adds an opt-in session GUC, `ec_ivf.tq_stage2_nprobe_cap`, that caps
the effective `ec_ivf.nprobe` only for persisted index-side TurboQuant stage-2
scans with final exact rerank enabled. The default `-1` preserves previous
behavior.

Code commit under review:

- `8c0246aec1af884659be4a397475c3c60f8a4833` - `Add TQ stage2 nprobe cap`

## What changed

- `src/am/ec_ivf/options.rs` registers `ec_ivf.tq_stage2_nprobe_cap`.
- `src/am/ec_ivf/scan.rs` applies the cap during scan rescan only when the
  index has a rerank sidecar, uses `coarse_rerank`, uses index-side TurboQuant
  rerank, and has an effective stage-2 final rerank width greater than zero.
- Unit tests cover matching TQ stage-2 scans and non-matching scans.

## Validation

- `cargo fmt --check`: passed
- `cargo test -p ecaz am::ec_ivf::scan::tests::tq_stage2_nprobe_cap --lib --no-default-features --features pg18`: passed, 2 tests
- `cargo check -p ecaz --lib --no-default-features --features pg18`: passed
- `cargo build --release -p ecaz`: passed
- `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`: passed
- `ecaz bench suite audit`: passed, 18 steps
- `ecaz bench suite run`: completed, 18 succeeded / 0 failed
- `ecaz bench suite status`: completed, 18 succeeded / 0 failed
- `ecaz bench suite report`: generated

Artifact source of truth:

- `artifacts/manifest.md`
- `artifacts/task124-tq-stage2-nprobe-cap-10-50-100-suite.json`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl`
- `artifacts/report-results.jsonl`
- packet-local recall, latency, storage logs under `artifacts/nprobe-cap-ab/`

## 10k / 50k / 100k A/B

Both variants use requested `nprobe=64`, `rerank_width=75`,
`rerank_group_width=50`, and `stage2_final_rerank_width=15`.

| Scale | Variant | Recall@10 | Latency p50 | Latency p95 | Latency p99 | Coarse candidates | TQ candidates | TQ scalar candidates |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | cap off | 1.0000 | 1.14 ms | 1.32 ms | 1.52 ms | 1,000,000 | 7,500 | 0 |
| 10k | cap 60 | 1.0000 | 1.09 ms | 1.25 ms | 1.37 ms | 936,366 | 7,500 | 0 |
| 50k | cap off | 0.9980 | 4.62 ms | 4.80 ms | 4.85 ms | 5,000,000 | 7,500 | 0 |
| 50k | cap 60 | 0.9980 | 4.56 ms | 4.90 ms | 5.50 ms | 4,525,933 | 7,500 | 0 |
| 100k | cap off | 1.0000 | 8.95 ms | 9.22 ms | 9.40 ms | 10,000,000 | 7,500 | 0 |
| 100k | cap 60 | 1.0000 | 8.59 ms | 8.85 ms | 9.03 ms | 9,556,278 | 7,500 | 0 |

Storage is unchanged by the session cap. The measured ec_ivf index sizes are:

- 10k: 10.9 MiB, 1143.6 B/row
- 50k: 50.9 MiB, 1066.8 B/row
- 100k: 100.8 MiB, 1057.2 B/row

## Outcome

This is a TQ-focused code change, not just a measurement packet. It proves the
TQ scorer path is fully SIMD/NEON in this fixture (`scalar_candidates=0`), and
the cap reduces coarse candidate work without changing recall.

It is not a Task 124 closeout. Latency is not uniformly better: 10k and 100k
improve, 50k mean/p50 improve slightly, but 50k p95/p99 regress. I recommend
treating this as a default-off tuning control and continuing Task 124 with a
deeper TQ scan/materialization optimization rather than claiming completion.
