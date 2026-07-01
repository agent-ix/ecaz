---
task: 125
topic: tq-scorer-optimization
requester: codex
date: 2026-07-01
code_commit: 9d8ce1da12
base_commit: 6799686af9e9adf13332bd4ec6e19b60e7ceb80e
---

# Review Request: TurboQuant Scorer Optimization

This packet covers the requested TurboQuant optimization tasks 125-129 as one scorer-path slice.

Implemented:

- Task 125/126: widened the LUT block width to 64 and reordered AVX2/NEON LUT scoring so each dimension table is loaded once per chunk and applied across octets.
- Task 128: reused caller-owned score buffers in IVF rerank/scoring paths and removed the extra estimates-negate pass for TurboQuant no-QJL paths.
- Task 129: no-QJL TurboQuant batch scoring no longer parses per-candidate gamma and avoids the redundant `Vec<&[u8]>` re-wrap in hot IVF payload paths.
- Task 127: enabled exact suffix-bound pruning for TurboQuant on NEON with sparse 512-dimension checkpoints. Non-NEON sessions fall back to the existing unbounded batch scorer.

Validation:

- `cargo check -p ecaz --lib`
- `cargo test -p ecaz --lib turboquant_lut_bounded_batch_keeps_and_prunes -- --test-threads=1`
- `cargo test -p ecaz --lib turboquant_dispatch_uses_score_bound_pruning -- --test-threads=1`
- `cargo test -p ecaz --lib lut32_ -- --test-threads=1`
- `cargo test -p ecaz --lib turboquant_no_qjl -- --test-threads=1`
- `cargo build --release -p ecaz`
- `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`
- `target/release/ecaz bench suite run ...` at 10k/50k/100k for recall, latency, and storage.

Evidence is in `artifacts/manifest.md`. Final A/B summary:

- 10k recall unchanged `0.9734`; latency mean improved `1.26 ms -> 1.22 ms`; storage unchanged at index `1028.1 B/row`.
- 50k recall unchanged `0.9521`; latency mean moved `2.55 ms -> 2.58 ms`; storage unchanged at index `964.7 B/row`.
- 100k recall unchanged `0.8969`; latency mean improved `3.91 ms -> 3.86 ms`; storage unchanged at index `948.2 B/row`.

Review focus:

- Confirm the widened LUT block/reordered table-load loops remain bit-exact and do not overrun tails.
- Confirm the no-QJL IVF/rerank paths correctly ignore gamma only for no-QJL and still require gamma for QJL.
- Confirm the NEON-only Task 127 activation/fallback split is appropriate and the 512-dimension checkpoint cadence is the right latency/early-prune tradeoff.
