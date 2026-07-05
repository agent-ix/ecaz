# Review Request: Task 28 IVF CLI Bench Profile and 10k Smoke

## Summary

This packet starts the IVF initial tuning lane with a narrow bench-enabling
slice:

- code checkpoint: `7e5a2b3`
- `ecaz-cli` now has an `ec_ivf` profile.
- `ecaz bench recall` / `latency` can sweep `ec_ivf.nprobe`.
- `ecaz corpus load` can plan a single IVF index with `nlists`, `nprobe`,
  `training_sample_rows`, `storage_format`, and `rerank` reloptions passed
  through by `--reloption`.
- IVF benchmark queries pass the ORDER BY probe as raw `real[]`, matching the
  current `ec_ivf` scan callback contract. HNSW and DiskANN retain encoded
  query probes.

DiskANN remains separate future work; this packet only touches the shared CLI
registry enough to name `ec_ivf`.

## Smoke Measurement

Local PG18 WSL smoke only, not a product benchmark.

Source fixture:

- source table: `ec_hnsw_parallel_concurrent_dsm_recall_corpus`
- copied smoke table: `task28_ivf_smoke10k_corpus`
- rows: 10,000
- dimensions after encoding source with `encode_to_ecvector(source, 4, 42)`: 64
- query rows: 20

Index:

```sql
CREATE INDEX task28_ivf_smoke10k_n64_idx
ON task28_ivf_smoke10k_corpus USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 64,
  nprobe = 64,
  training_sample_rows = 10000,
  storage_format = 'turboquant',
  rerank = 'off'
);
```

Observed locally:

| metric | result |
|---|---:|
| build time | `00:05.823` |
| index size | `1,236,992` bytes (`1208 kB`) |
| heap size | `6,291,456` bytes (`6144 kB`) |
| full-probe EXPLAIN execution time, one query | `38.212 ms` |
| full-probe candidates scored | `10,000` |

Tiny recall smoke versus exact compressed indexed-row scan:

| nprobe | returned | exact hits | recall@10 |
|---:|---:|---:|---:|
| 1 | 200 | 142 | 0.7100 |
| 4 | 200 | 142 | 0.7100 |
| 16 | 200 | 142 | 0.7100 |
| 64 | 200 | 142 | 0.7100 |

Every query returned 10 rows at every tested `nprobe`.

Interpretation: this is only a smoke baseline proving the `ec_ivf` catalog,
build, scan, and packet-local measurement path work on an existing real-corpus
surface. The flat nprobe curve needs follow-up: either this 20-query / 64-dim
smoke is too small to expose routing differences, or the SQL shape needs a
stronger plan check per sweep point before we treat nprobe as measured.

## Artifacts

- `artifacts/pg18-ivf-10k-smoke.sql`
- `artifacts/pg18-ivf-10k-smoke.log`
- `artifacts/pg18-corpus-inspect.log`
- `artifacts/pg18-corpus-schema.log`
- `artifacts/pg18-extension-am-check.log`
- `artifacts/pg18-install-ivf-catalog.sql`
- `artifacts/pg18-install-ivf-catalog.log`
- `artifacts/pg18-version-smoke.log`
- `artifacts/manifest.md`

## Validation

- `cargo test -p ecaz-cli`
- `git diff --check`

PG18 only, per current task direction.

## Next Slice Recommendation

Use this CLI profile to run the first real Phase 8 bench packet on a larger
surface:

1. Rebuild IVF on the 50k corpus with `nlists` in `{64, 128, 256}`.
2. For each index, run EXPLAIN-backed sweeps for `nprobe` in `{1, 4, 16, full}`
   and record selected-list counts so GUC behavior is proven per point.
3. Capture recall, latency percentiles, index size, and build time in one
   packet-local artifact set before optimizing the scan path further.
