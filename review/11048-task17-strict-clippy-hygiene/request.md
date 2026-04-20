# Review Request: Task 17 Strict Clippy Hygiene

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `src/am/ec_diskann/reader.rs`
- `src/am/ec_diskann/scan.rs`
- `src/am/ec_diskann/vamana.rs`
- `src/am/ec_diskann/vacuum.rs`

## What this packet is

This is the dedicated `ec_diskann` strict-clippy cleanup packet that reviewer
feedback on 11045 explicitly asked to land before treating task 17 as cleanly
closed.

Before this packet, `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
still failed on the same carried-forward `ec_diskann` baseline:

- `unnecessary_sort_by` in `reader.rs`, `scan.rs`, and `vamana.rs`
- test-only `unnecessary_cast` / `needless_borrows_for_generic_args` in `scan.rs`
- test-only `needless_range_loop` in `vacuum.rs`

After this packet, the strict clippy gate is green.

## Why this slice

The per-slice deferment was reasonable while callback wiring was still moving,
but by the end of Phase 9 the accumulated `ec_diskann` warnings had become
task-17 debt rather than harmless local noise.

This packet pays down exactly that debt without widening scope into unrelated
algorithm changes.

## What changed

### `reader.rs`

Replaced `sort_by(|a, b| a.cmp(b))` with `sort()` at the two frontier-sort
sites.

### `scan.rs`

- Replaced the two `sort_by(|a, b| a.cmp(b))` calls with `sort()`
- Removed redundant same-type casts in the scan tests
- Removed needless borrows when passing the test closures into `vamana_scan`

### `vamana.rs`

Replaced the four `sort_by(|a, b| a.cmp(b))` sites with `sort()`.

### `vacuum.rs`

Rewrote the small test helper loop in `vc_009_repair_preserves_encoded_length`
to use `iter_mut().enumerate().take(6)` instead of indexing a vector through a
range loop.

No runtime behavior or planner behavior changed in this packet.

## Boundary after this packet

This closes the outstanding strict-clippy hygiene debt inside
`src/am/ec_diskann/*`.

Still not closed here:

- the real-10k Recall@10 artifact packet requested by review 11045

That remaining item is environment-dependent rather than code-local: this
workspace did not have a staged DBpedia parquet / TSV fixture on disk, and the
scratch pg17 cluster was not running when I checked, so there was no honest
real-corpus surface to measure against in this packet.

## Verification

```text
cargo fmt -- src/am/ec_diskann/reader.rs src/am/ec_diskann/scan.rs src/am/ec_diskann/vamana.rs src/am/ec_diskann/vacuum.rs
cargo build --lib
cargo clippy --lib --no-deps
cargo test --lib ec_diskann
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Observed:

- `cargo fmt -- src/am/ec_diskann/reader.rs src/am/ec_diskann/scan.rs src/am/ec_diskann/vamana.rs src/am/ec_diskann/vacuum.rs` — passed
- `cargo build --lib` — passed
- `cargo clippy --lib --no-deps` — passed
- `cargo test --lib ec_diskann` — passed with `143 passed`, `0 failed`
- `cargo test` — passed with `660 passed`, `0 failed`, `4 ignored`
- `bash scripts/run_pgrx_pg17_test.sh` — passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings` — passed

## Reviewer notes

- This packet is intentionally boring: it is lint-only hygiene for existing
  `ec_diskann` code and tests.
- The important outcome is not the specific `sort()` rewrites; it is that
  task 17 no longer carries a known-failing strict-clippy baseline inside
  `src/am/ec_diskann/*`.
- Recall measurement remains a separate packet because the signoff surface
  must be a real staged corpus, not a guessed or synthetic substitute.
