# Post-Main RaBitQ / IVF / SPIRE Sweep

## Scope

This packet records the Task 50 branch after the Task 39 / Task 47 upstream
merge and the follow-on SPIRE DML unsafe cleanup:

- `69269be5a` merges `origin/main` at `24e7ea814`.
- `465d83def` consolidates the SPIRE DML predicate boundary.
- `3a1cbe69c` removes stale SPIRE DML test unsafe wrappers.
- `14d800937` adds the post-merge review packet set already reviewed in this
  packet's feedback.

The validation sweep is intentionally focused on the AWS optimization prep lane:
RaBitQ, IVF, and SPIRE. HNSW and DiskANN are out of scope for this sweep.

## Result Summary

- Merge from `origin/main` was clean and pushed on `task-50-unsafe-closeout`.
- `cargo test --no-run --all-targets --no-default-features --features pg18,bench`
  completed successfully. The only warning observed is the existing SPIRE DML
  re-export warning in `src/am/mod.rs`.
- Direct filtered Rust and pgrx RaBitQ test execution did not reach assertions
  because the local test binaries failed dynamic symbol lookup:
  `pg_re_throw` for direct `cargo test` and `LockBuffer` for `cargo pgrx test`.
- Local PG18 scratch corpus inventory found prepared 10k IVF/RaBitQ and
  SPIRE/RaBitQ fixtures with queries and indexes.
- `ecaz bench suite` audit, dry-run, run, status, and report completed for four
  local smoke-sized RaBitQ/IVF/SPIRE steps.
- SPIRE CustomScan read smoke passed.
- SPIRE multicluster base smoke failed because remote search executor
  `endpoint_status` reports `requires_rabitq_storage_format` is not ready.
- SPIRE insert/read-after-CustomScan probes failed because local heap tuple
  delivery still requires `custom_scan_tuple_delivery` before consuming remote
  placements.

## Bench Snapshot

Suite config:
`reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/rabitq-ivf-spire-local-suite.json`

Suite report:
`reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/ecaz-bench-suite-report.md`

Key local PG18 results on `tqvector_bench` at `localhost:28818`:

| Lane | nprobe | recall@10 | mean query/latency | p95 latency |
| --- | ---: | ---: | ---: | ---: |
| IVF RaBitQ recall | 8 | 0.9720 | 59.36 ms | n/a |
| IVF RaBitQ recall | 16 | 0.9780 | 97.70 ms | n/a |
| IVF RaBitQ latency | 8 | n/a | 62.1 ms | 82.0 ms |
| IVF RaBitQ latency | 16 | n/a | 92.2 ms | 119.6 ms |
| SPIRE RaBitQ recall | 8 | 0.9880 | 330.34 ms | n/a |
| SPIRE RaBitQ recall | 16 | 0.9960 | 416.22 ms | n/a |
| SPIRE RaBitQ latency | 8 | n/a | 229.7 ms | 286.8 ms |
| SPIRE RaBitQ latency | 16 | n/a | 411.1 ms | 509.0 ms |

## Artifacts

See `artifacts/manifest.md` for command lines, timestamps, fixture details, and
the packet-local evidence list.
