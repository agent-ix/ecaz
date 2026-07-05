# Review Request: DiskANN load preflights single-index rebuild conflicts

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/commands/corpus/load.rs`

## What this packet is

DiskANN uses one corpus-index name per prefix (`<prefix>_idx`, or
`<prefix>_<storage_format>_idx`). If an operator reran
`ecaz corpus load --profile ec_diskann ...` with different reloptions,
the CLI previously fell through to raw Postgres failure:

- existing index name did not match the requested reloptions, so the
  idempotent skip path did not trigger
- `CREATE INDEX` then failed with a generic "already exists" error

This packet turns that into an explicit DiskANN-facing preflight that
explains why the rebuild cannot happen in place and what the operator
should do next.

## What changed

### `crates/ecaz-cli/src/commands/corpus/load.rs`

- Added a pure formatter for the single-index conflict case:

```rust
fn existing_single_index_conflict_error(
    profile: &IndexProfile,
    index: &str,
    reloptions: &[(String, String)],
) -> String
```

- The message includes:
  - the blocking index name
  - the selected profile (`ec_diskann`)
  - a direct `DROP INDEX ...` example
  - the requested reloptions in normalized `key=value` form

- `ensure_index(...)` now checks for the non-HNSW single-index conflict
  before issuing `CREATE INDEX`:

```rust
if !profile.sweep_axis_is_m() && psql::relation_exists(client, &job.name, 'i').await? {
    return Err(eyre!(existing_single_index_conflict_error(
        profile,
        &job.name,
        &job.reloptions
    )));
}
```

- Added `existing_single_index_conflict_error_points_diskann_operator_at_drop_index`
  to pin the operator-facing guidance for a retuned DiskANN build.

## Why this slice

- It fixes a real DiskANN load failure mode, not a cosmetic label issue.
- It keeps scope tight to one file and one preflight branch.
- It complements packets 11066 and 11067 by making the main DiskANN load
  path fail loudly and specifically instead of surfacing generic errors.

## Test evidence

```text
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 204 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also ran locally for this slice on `pg18`:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

## Follow-ups intentionally not in this packet

- Auto-dropping or auto-renaming existing DiskANN indexes. This packet
  only adds a clear preflight error; it does not make destructive
  changes on the operator's behalf.
- Extending the same preflight to every HNSW reloption mismatch shape.
  This slice stays on the DiskANN single-index path that currently
  produces the worst operator experience.
- Reading the existing index's reloptions back from Postgres and printing
  a requested-vs-existing diff. The requested reloptions are enough to
  explain the operator action needed here.
