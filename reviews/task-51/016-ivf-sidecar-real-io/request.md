# Review Request: IVF Sidecar Real-I/O Modes

## Scope

Code commit under review:

- `0b359e5ddbee42a7cba45042f7da577d1accf7d4` - real-I/O sidecar rerank modes

Benchmark packet:

- `benchmarks/task51-local-ivf-sidecar-real-io/`

This slice extends the existing `ecaz bench sidecar-rerank` harness so the
measurement can distinguish the original free-I/O upper bound from two
separate-table read modes:

- `free`: existing in-process resident sidecar bytes.
- `random-id`: one primary-key sidecar table lookup per candidate id.
- `tid-sorted`: one batched sidecar table fetch ordered by physical `ctid`.

The sidecar tables are unlogged fixed-width `bytea` tables with `payload`
stored `PLAIN`, so this measures real PostgreSQL table reads without using
`real[]` or toast-backed source vectors.

## Result

Local validation:

```text
cargo test -p ecaz-cli sidecar
git diff --check
```

Both completed successfully. The test run passed 7 focused CLI tests, including
the sidecar read-mode and suite expansion coverage.

Benchmark status:

```text
[suite:task51-local-ivf-sidecar-real-io] completed=1 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Main benchmark finding:

- Random-id lookup adds about 17-18 ms p50 sidecar I/O for 50 candidates.
- TID-sorted batch fetch adds about 0.9-1.4 ms p50 sidecar I/O.
- F16 preserves candidate-frontier recall in this fixture and reaches recall@10
  `0.9980` by nprobe 96/128.
- RaBitQ8 remains much smaller but recall-limited at this candidate width.

## Notes For Review

- This addresses the packet 008 reviewer feedback that the previous sidecar
  harness was only a free-I/O oracle.
- The code remains a measurement harness; it does not introduce an in-index
  product sidecar storage format.
- The suite intentionally did not run AWS, vchord, or pgvectorscale.
- The TID-sorted mode is a local microbenchmark of separate-table physical-order
  reads. It is not the final product design.

See `artifacts/manifest.md` for packet-local validation artifacts.
