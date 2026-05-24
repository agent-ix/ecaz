# Task 51 Packet 003 Artifact Manifest

- task bucket: `reviews/task-51/003-rabitq-byte-lut-state`
- code commit under review: `e9933e6b7431f24568ac49efd2c249e5c78efa9f`
- timestamp: `2026-05-22T21:58:52-07:00`
- surface: local PG18 only; no AWS
- lane: RaBitQ prepared-query state used by `ec_ivf`
- storage format: `rabitq`
- quant bits covered by smoke: `1` build path and `4` build + scan path
- isolated one-index-per-table smoke surface: yes

## Artifacts

- `cargo-check-pg18.log`
  - command: `cargo check --lib --no-default-features --features pg18`
  - result: passed
  - key line: `Finished dev profile`
  - note: existing unrelated warnings remain in `src/am/mod.rs` and `src/am/ec_ivf/build.rs`.
- `cargo-test-no-run-prepared-byte-lut.log`
  - command: `cargo test --lib prepared_queries_only_keep_bits1_byte_lut_for_bits1 --no-run --no-default-features --features pg18`
  - result: passed
  - key line: `Finished test profile`
  - note: `--no-run` is used because this checkout's lib-test executable has an existing runtime PostgreSQL symbol startup issue; this still verifies the new test compiles.
- `rustfmt-rabitq.log`
  - command: `rustfmt --check src/quant/rabitq.rs`
  - result: passed
  - note: rustfmt emitted existing stable-channel warnings for unstable config keys.
- `git-diff-check.log`
  - command: `git diff --check -- src/quant/rabitq.rs`
  - result: passed
- `cargo-pgrx-install-pg18.log`
  - command: `cargo pgrx install --test --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
  - result: passed
  - key line: `Finished installing ecaz`
- `run-pg18-ivf-rabitq-byte-lut-smoke.sh`
  - starts an isolated temporary PG18 cluster with `shared_preload_libraries=ecaz`
  - builds an `ec_ivf` RaBitQ index with `quant_bits = 1`
  - drops it, then builds an `ec_ivf` RaBitQ index with `quant_bits = 4`
  - runs `EXPLAIN (ecaz, ANALYZE, COSTS OFF, VERBOSE)` against the bits=4 index
- `pg18-ivf-rabitq-byte-lut-smoke.log`
  - command: `bash reviews/task-51/003-rabitq-byte-lut-state/artifacts/run-pg18-ivf-rabitq-byte-lut-smoke.sh`
  - result: passed
  - key lines:
    - `shared_preload_libraries | ecaz`
    - `CREATE INDEX` for the bits=1 and bits=4 index builds
    - `Rerank Rows: 3`
    - `Heap Blocks Fetched: 1`
    - `Execution Time: 0.483 ms`
