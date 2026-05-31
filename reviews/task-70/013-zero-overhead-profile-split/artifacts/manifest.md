# Task 70 Packet 013 Artifact Manifest

- Code commit: `261186cf7ee217ab51e5b061c1ee8e5e1c8c95bc`
- Task bucket: `reviews/task-70/`
- Packet path: `reviews/task-70/013-zero-overhead-profile-split/`
- Timestamp: `2026-05-31T21:58:49Z`
- Lane: Task 70 follow-up from packet 012 review, zero-overhead default profile split
- Fixture: real10K DBPedia staged corpus
- Storage format / rerank mode: `ec_diskann`, `pq_fastscan`, `graph_degree=32`, `build_list_size=100`, `alpha=1.2`, `rerank_budget=64`, `top_k=10`
- Isolation: one packet-local table/index prefix, `task70_013_diskann`; pgvectorscale compare rebuilds a separate packet-local compare table
- Runner: `ecaz bench suite`

## Validation Commands

```sh
cargo fmt --check > reviews/task-70/013-zero-overhead-profile-split/artifacts/cargo-fmt-check.log 2>&1
cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests:: > reviews/task-70/013-zero-overhead-profile-split/artifacts/cargo-test-diskann-scan.log 2>&1
cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings > reviews/task-70/013-zero-overhead-profile-split/artifacts/cargo-clippy-pg18.log 2>&1
```

## Install Command

```sh
./target/debug/ecaz dev install ecaz-pg-test --pg 18 --database tqvector_bench --log-file reviews/task-70/013-zero-overhead-profile-split/artifacts/install-ecaz-pg-test.log
```

Installed backend SHA256: `7c5d3814e7699a95b0556470d36d77bd3ecc70b6f1437a6c52482fd65abd2025`.

## Suite Commands

Dry run:

```sh
./target/debug/ecaz bench suite run --config reviews/task-70/013-zero-overhead-profile-split/artifacts/suite.json --dry-run --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-70/013-zero-overhead-profile-split/artifacts/suite-dry-run-manifest.json --log-file reviews/task-70/013-zero-overhead-profile-split/artifacts/suite-dry-run.log
```

Full run:

```sh
./target/debug/ecaz bench suite run --config reviews/task-70/013-zero-overhead-profile-split/artifacts/suite.json --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-70/013-zero-overhead-profile-split/artifacts/suite-manifest.json --results-output reviews/task-70/013-zero-overhead-profile-split/artifacts/results.jsonl --log-file reviews/task-70/013-zero-overhead-profile-split/artifacts/suite-run.log
```

## Artifacts

| artifact | command / source | key result |
| --- | --- | --- |
| `suite.json` | Checked-in `SuiteConfig` for packet 013 | Defines load, recall, and clean pgvectorscale compare at L64/L200. |
| `suite-dry-run.log`, `suite-dry-run-manifest.json` | Dry-run command above | Dry run succeeded and kept expected outputs under this packet. |
| `install-ecaz-pg-test.log` | Install command above | Installed PG18 extension for code commit `261186cf7`. |
| `suite-run.log`, `suite-manifest.json`, `results.jsonl` | Suite command above | Suite succeeded; all selected steps exit code 0. |
| `load-diskann-real10k.log` | Suite load step | copy corpus 5.69s, encode corpus 2.38s, build index 7.12s, total 25.34s. |
| `recall-diskann-real10k-l64-l200.log` | Suite recall step | L64 recall 0.9965, mean q-time 0.63 ms; L200 recall 0.9975, mean q-time 0.79 ms. |
| `compare-vectorscale-real10k-l64-l200.log` | Suite pgvectorscale compare step | L64 ec_diskann 0.66 ms mean / 0.9965 recall vs pgvectorscale 0.63 ms / 0.9955; L200 ec_diskann 0.81 ms / 0.9975 vs pgvectorscale 1.22 ms / 1.0000. |
| `truth-real10k-k10.json` | Recall truth cache | Packet-local ground truth cache for k=10. |
| `summary.md` | Manual aggregation from `results.jsonl` and validation logs | Summary tables and interpretation for review. |
| `cargo-fmt-check.log` | Validation command above | Finished successfully. |
| `cargo-test-diskann-scan.log` | Validation command above | 20 scan tests passed. |
| `cargo-clippy-pg18.log` | Validation command above | Finished successfully. |

## Checksums

```text
8af9ca6eccdc7a97ddc763e8049b661b5363eede  cargo-clippy-pg18.log
9e151839b689a235d1759b2e9ba931442182d350  cargo-fmt-check.log
4696df03f2db3131a3d0dc4a89d30a9a7e78a2a6  cargo-test-diskann-scan.log
305cecfd15c8e58657c631d9e2b4f9714872acc7  compare-vectorscale-real10k-l64-l200.log
885879f08fe4fa57a4001c3ebc99f53053969a9e  install-ecaz-pg-test.log
84726880536c0c752f772c5a6225ad517f7e2920  load-diskann-real10k.log
2b5e5c26d1fba95ee681b9ab3f3a84d07de4512d  recall-diskann-real10k-l64-l200.log
c246bed6effe339c3fae1dac2911297d95dac8ab  results.jsonl
73c9cbbaacbcba38153eb6e6c459332d5d6f5791  suite-dry-run-manifest.json
f1fbe0be5d880c641403860a5b6589f2f9fbaf93  suite-dry-run.log
129701982dff96ce2627872738f164d3829c9a60  suite-manifest.json
55cf260469178e31a7813e6527cbf08c724cd947  suite-run.log
1e7338df67d24eab72e444cb7e6af2a3cd04f042  suite.json
841de9ed91811e825499494d6890cd3061c7c62b  truth-real10k-k10.json
```
