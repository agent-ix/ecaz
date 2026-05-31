# Task 70 / Packet 013: Zero-Overhead Profile Split

## Packet Scope

- Code commit: `261186cf7ee217ab51e5b061c1ee8e5e1c8c95bc`
- Review driver: packet `012` reviewer follow-up
- Summary: `artifacts/summary.md`
- Manifest: `artifacts/manifest.md`

This packet addresses the non-blocking packet 012 concern that the unprofiled scan path still paid runtime `Option<&mut FrontierProfile>` checks in the frontier inner loop.

## Code Change

`src/am/ec_diskann/scan.rs` now keeps the default and profiled paths separate:

- `vamana_scan_with` calls the branch-free `greedy_descent_with`.
- `vamana_scan_with_frontier_profile` calls `greedy_descent_with_frontier_profile`.
- `greedy_descent_with` has no `FrontierProfile` parameter, no `Option<&mut FrontierProfile>`, and no profile branches in the inner loop.
- The profiled variant keeps the timing/counter instrumentation unconditionally.
- Shared validation and rerank finalization remain extracted outside the frontier hot loop.

No new `unsafe` blocks were introduced.

## Validation

- `cargo fmt --check`: pass
- `cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::`: pass, 20/20
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`: pass

## Measurement

Packet-local suite:

```sh
./target/debug/ecaz bench suite run --config reviews/task-70/013-zero-overhead-profile-split/artifacts/suite.json --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-70/013-zero-overhead-profile-split/artifacts/suite-manifest.json --results-output reviews/task-70/013-zero-overhead-profile-split/artifacts/results.jsonl --log-file reviews/task-70/013-zero-overhead-profile-split/artifacts/suite-run.log
```

Key results:

- Recall: L64 `0.9965`; L200 `0.9975`.
- Clean compare L64: `ec_diskann` `0.66 ms` mean / `1.10 ms` p99 vs pgvectorscale `0.63 ms` / `1.04 ms`.
- Clean compare L200: `ec_diskann` `0.81 ms` mean / `1.11 ms` p99 vs pgvectorscale `1.22 ms` / `1.81 ms`.
- Packet 012 acceptance target was L200 mean `<= 0.83 ms`; this packet measured `0.81 ms`.

## Review Ask

Please review whether this resolves the packet 012 follow-up:

1. The default scan path is free of profile-option branches in the frontier inner loop.
2. The profiled path still records frontier counters and preserves scan results.
3. The L200 clean compare regression is closed against the `<= 0.83 ms` target.
