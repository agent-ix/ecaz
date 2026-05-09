# Review Request: SPIRE Adaptive Nprobe Treatment

Checkpoint: `31f6f302` (`Add SPIRE adaptive nprobe treatment`)

## Scope

- Adds default-off SPIRE adaptive `nprobe` controls:
  - `ec_spire.adaptive_nprobe`
  - `ec_spire.adaptive_nprobe_score_gap_micros`
- Implements a deterministic threshold policy that halves the requested routing
  width only when the score gap at the retained frontier boundary clears the
  configured threshold.
- Applies the policy consistently to flat, recursive, and top-graph route
  selection while keeping the existing configured path unchanged when disabled.
- Extends `ec_spire_index_scan_routing_snapshot` with
  `adaptive_nprobe_decision` and per-level effective `nprobe` diagnostics.
- Adds SPIRE-only `ecaz bench recall/latency` flags:
  `--adaptive-nprobe` and `--adaptive-nprobe-score-gap-micros`.
- Updates the Phase 9 and Phase 10 task files:
  - Phase 9.7 adaptive `nprobe` is marked complete.
  - Phase 10.3 rerank-width measurement is marked complete from baseline plus
    treatment packets.
  - Phase 10.7 records adaptive bench flags as partial harness work while
    keeping route-budget and remote-fanout coverage open.

## Local Treatment Result

Local real10k, prefix `task30_p9_quality_base_c5ed545`, `nprobe=16`,
`rerank_width=50`, first 100 query rows:

| mode | recall@10 | NDCG@10 | mean q-time | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| control | 1.0000 | 1.0000 | 118.66 ms | 117.1 ms | 122.7 ms | 131.6 ms |
| adaptive gap150000 | 1.0000 | 1.0000 | 115.18 ms | 115.9 ms | 121.4 ms | 125.4 ms |

The rw25 tuning runs also found recall-safe thresholds, but latency was noisy;
the rw50 gap150000 treatment is the cited result because it preserves recall and
improves mean/p50/p95/p99 against a same-build control.

## Validation

- `cargo test --no-default-features --features pg18 --lib --no-run`
- `cargo test --no-default-features --features pg18 am::ec_spire::scan::tests::adaptive_nprobe_reduces_routing_width_when_boundary_gap_is_large --lib -- --exact`
- `cargo test --no-default-features --features pg18 am::ec_spire::scan::tests::adaptive_nprobe_keeps_configured_width_when_boundary_gap_is_small --lib -- --exact`
- `cargo test -p ecaz-cli adaptive_nprobe`
- `cargo fmt --check`
- `git diff --check`
- `target/debug/ecaz bench recall ...` treatment/control runs recorded under
  `artifacts/`
- `target/debug/ecaz bench latency ...` treatment/control runs recorded under
  `artifacts/`

Packet-local PG test rerun note: the new `pg_test` for
`ec_spire_index_scan_routing_snapshot` compiled, but the packet-local rerun was
blocked when pgrx tried to install into `/home/peter/.pgrx` from the sandbox
(`Read-only file system`). The packet includes that failed rerun log rather than
claiming it as successful validation.

## Review Focus

- Check that the adaptive policy is truly default-off and deterministic.
- Check that recursive/top-graph routing uses the same selected routes for
  production and diagnostics.
- Check the new SQL diagnostic column names and values for operator usefulness.
- Check that the CLI flags are narrow enough for SPIRE and reproducible enough
  for future treatment packets.
