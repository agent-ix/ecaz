# Task 28 IVF Borrowed Posting Scan Recall Check

This packet adds the recall check for the borrowed posting scan path from
packet 30069 and commit `30d9ffc`. It also records commit `86df0a2`, which adds
`ecaz bench recall --log-output` so recall packets can store raw tables without
shell redirection.

## Measurement Result

Fixture:

- Local PG18 scratch database `postgres`.
- Existing isolated n64 DBPedia-derived surfaces from packet 30052:
  - `task28_ivf_postopt10k_n64w25`
  - `task28_ivf_postopt25k_n64w25`
- `ecaz bench recall`, profile `ec_ivf`, `k=10`, `queries-limit=100`, sweep
  `nprobe=32,48`.
- Cache state: warm local development run; no explicit cache drop.
- Memory high-water mark: not captured.

| surface | nprobe | packet 30052 recall@10 | new recall@10 | ndcg@10 | mean q-time |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k n64 w25 | 32 | 0.9800 | 0.9800 | 0.9981 | 93.67 ms |
| 10k n64 w25 | 48 | 1.0000 | 1.0000 | 1.0000 | 134.17 ms |
| 25k n64 w25 | 32 | 0.9840 | 0.9840 | 0.9988 | 233.10 ms |
| 25k n64 w25 | 48 | 0.9990 | 0.9990 | 1.0000 | 330.18 ms |

The borrowed posting scan path preserved the previously measured recall at
these n64 operating points while packet 30069 showed lower p50 latency.

## Artifacts

- `artifacts/recall_10k_n64w25_nprobe32_48.log`
- `artifacts/recall_25k_n64w25_nprobe32_48.log`
- `artifacts/manifest.md`

## Validation

For the landed recall logging support:

- `cargo fmt --check`
- `cargo test -p ecaz-cli recall`
- `git diff --check`

The scan-path validation is recorded in packet 30069:

- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_heap_f32`

## Recommendation

Treat packet 30069 plus this recall check as the first useful posting-tuple
handling improvement after the pre-rerank top-k cleanup. The next IVF slice
should target denser posting-list layout or lower scored-posting volume rather
than more rerank-width tuning. DiskANN remains task 29 and is not included.
