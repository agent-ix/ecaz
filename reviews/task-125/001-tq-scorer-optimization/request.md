---
task: 125
topic: tq-scorer-optimization
requester: codex
date: 2026-07-01
code_commit: 96782e209
base_commit: 6799686af9e9adf13332bd4ec6e19b60e7ceb80e
---

# Review Request: TurboQuant Scorer Optimization

This packet covers the requested TurboQuant optimization tasks 125-129 as one scorer-path slice.

Implemented:

- Task 125/126: widened the LUT block width to 64 and added a NEON batch-tiled scorer that walks dimension chunks across every candidate octet in the flush before moving to the next chunk.
- Task 125: compacted the explicit no-QJL 4-bit query LUT from `f32` to `i16 + scale`; NEON scores the compact table with integer accumulators and applies the scale once per lane.
- Task 128: reused caller-owned score buffers in IVF rerank/scoring paths and removed the extra estimates-negate pass for TurboQuant no-QJL paths.
- Task 129: no-QJL TurboQuant batch scoring no longer parses per-candidate gamma and avoids the redundant `Vec<&[u8]>` re-wrap in hot IVF payload paths.
- Task 127: enabled exact suffix-bound pruning for TurboQuant on NEON with sparse 512-dimension checkpoints and added kept/pruned counter reporting to the block-kernel snapshots.

Validation:

- `cargo check -p ecaz --lib`
- `cargo test -p ecaz --lib turboquant_lut_bounded_batch_keeps_and_prunes -- --test-threads=1`
- `cargo test -p ecaz --lib turboquant_dispatch_uses_score_bound_pruning -- --test-threads=1`
- `cargo test -p ecaz --lib lut32_ -- --test-threads=1`
- `cargo test -p ecaz --lib lut32_tiled_batch_matches_scalar_tail_bits_across_widths_and_dims -- --test-threads=1`
- `cargo test -p ecaz --lib explicit_lut -- --test-threads=1`
- `cargo test -p ecaz --lib turboquant_no_qjl -- --test-threads=1`
- `cargo test -p ecaz-cli block_kernel_counter_lines_include_transition_formats -- --test-threads=1`
- `cargo build --release -p ecaz`
- `cargo build --release -p ecaz-cli`
- `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`
- `target/release/ecaz bench suite run ...` at 10k/50k/100k for recall, latency, and storage.
- `ECAZ_TQ_BATCH_WIDTH_PROFILE_CANDIDATES=20000 cargo test -p ecaz --lib task124_profile_tq_no_qjl_flush_widths -- --ignored --nocapture --test-threads=1`

Evidence is in `artifacts/manifest.md`. Final accepted int16 compact-LUT summary versus the prior sparse Task 127 candidate:

- 10k recall unchanged `0.9734`; latency mean improved `1.22 ms -> 0.88 ms`; TurboQuant NEON kernel improved `46.240597 ms -> 23.924550 ms`; storage unchanged at index `1028.1 B/row`.
- 50k recall unchanged `0.9521`; latency mean improved `2.58 ms -> 1.88 ms`; TurboQuant NEON kernel improved `76.955763 ms -> 41.573022 ms`; storage unchanged at index `964.7 B/row`.
- 100k recall unchanged `0.8969`; latency mean improved `3.86 ms -> 2.73 ms`; TurboQuant NEON kernel improved `74.883089 ms -> 38.896097 ms`; storage unchanged at index `948.2 B/row`.

Closeout evidence added after the audit:

- Task 125 literal tiling: `score_lut_no_qjl_4bit_batch_tiled` now routes normal no-QJL flushes through a NEON dim-tile -> all-candidate-octets loop, with scalar/block fallback for other ISAs.
- Task 126 width curve: debug microprofile reports width 32/64/128 at `12662.2`, `11578.1`, and `11084.1 ns/candidate` respectively in `artifacts/task126-width-profile.log`.
- Task 127 prune fraction reporting: release latency logs now include `pruned_candidates` / `kept_candidates`; the 10k/50k/100k suite observed no bounded dispatch for this IVF configuration (`0/0`), and the focused bounded scorer test verifies both all-kept and all-pruned accounting.

Review focus:

- Confirm the widened/compact LUT block loops remain bit-exact and do not overrun tails.
- Confirm the no-QJL IVF/rerank paths correctly ignore gamma only for no-QJL and still require gamma for QJL.
- Confirm the NEON-only Task 127 activation/fallback split is appropriate and the 512-dimension checkpoint cadence is the right latency/early-prune tradeoff.
