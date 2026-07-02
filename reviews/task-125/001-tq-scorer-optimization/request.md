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
- Task 127 prune fraction reporting: standard non-heap IVF still observes no bounded dispatch (`0/0`) because it has no running top-k cutoff, so the closeout reran TurboQuant IVF with `rerank=heap_f32,rerank_width=100`. The production heap-rerank logs report exact prune-on/prune-off recall parity and nonzero bounded dispatch at 10k/50k/100k:
  - 10k: recall `1.0000 -> 1.0000`; latency mean `1.50 ms`; `lut32_pruned_candidates=186274`, `lut32_kept_candidates=3301` (`98.3%` pruned among bounded candidates).
  - 50k: recall `0.9641 -> 0.9641`; latency mean `2.58 ms`; `lut32_pruned_candidates=320421`, `lut32_kept_candidates=7698` (`97.7%` pruned among bounded candidates).
  - 100k: recall `0.9268 -> 0.9268`; latency mean `3.80 ms`; `lut32_pruned_candidates=316516`, `lut32_kept_candidates=7049` (`97.8%` pruned among bounded candidates).

Review focus:

- Confirm the widened/compact LUT block loops remain bit-exact and do not overrun tails.
- Confirm the no-QJL IVF/rerank paths correctly ignore gamma only for no-QJL and still require gamma for QJL.
- Confirm the NEON-only Task 127 activation/fallback split is appropriate and the 512-dimension checkpoint cadence is the right latency/early-prune tradeoff.
