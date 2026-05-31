# Task 70 / Packet 001: Scan Profile Notice

## Code Under Review

- Commit: `a93e464024818b64bd18b0b438d2652dc0964d8f`
- Scope: adds an opt-in `ec_diskann.scan_profile_notice` GUC and a profiled `ec_diskann` scan execution path used only when that GUC is enabled.

## Why

Task 70 Phase 1 needs a repeatable scan-path split before ranking P0 scan-kernel slices. The normal `ec_diskann` scan path already supports the binary-sidecar fast path, but it did not emit durable phase-level timing. This packet adds a developer measurement switch that emits one NOTICE per `amrescan`:

```text
ec_diskann_scan_profile list_size=... rerank_budget=... top_k=... binary_sidecar=... setup_us=... entry_resolution_us=... graph_read_decode_us=... prefilter_score_us=... frontier_us=... heap_prefetch_us=... exact_rerank_us=... result_expand_us=... total_us=... graph_read_count=... prefilter_count=... rerank_count=... result_count=...
```

The default remains off, so production and benchmark paths do not pay the additional `Instant` / `RefCell` profiling overhead unless explicitly enabled.

## Implementation Notes

- `src/am/ec_diskann/options.rs` registers `ec_diskann.scan_profile_notice` as a `USERSET` bool GUC with default `false`.
- `src/am/ec_diskann/routine.rs` keeps the existing unprofiled `execute_diskann_scan` path when the GUC is off.
- When enabled, the scan wraps the `GraphReader`, prefilter closure, prefetch closure, and rerank closure to accumulate phase timings and event counts.
- `setup_us` is kept separate from `entry_resolution_us`; `frontier_us` is the residual scan-shell time after graph read/decode, prefilter, prefetch, and exact rerank are subtracted from the profiled `vamana_scan_with` wall time.
- No new `unsafe` blocks.

## Validation

See `artifacts/manifest.md`.

Commands run:

- `cargo check --all-targets --no-default-features --features pg18`
- `cargo fmt --check`
- `cargo test --no-default-features --features pg18 scan_profile_notice_guc_defaults_to_off`
- `cargo test --no-default-features --features pg18 sc_011_scan_with_scratch_reuse_matches_fresh`

## Follow-Up

Use this packet to run Task 70 Phase 1 on the M5 real10K fixture at L=64 and L=200 with `SET ec_diskann.scan_profile_notice = on`, capture NOTICE output packet-locally, and combine it with `EXPLAIN (ANALYZE, BUFFERS)` / external profiler evidence before ranking P0 slices.
