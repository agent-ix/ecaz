# Review Request: Show CLI profile names in `ecaz bench storage`

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/commands/bench/storage.rs`

## What this packet is

`ecaz bench storage` still printed raw access-method names in its
per-index table, even after task-17 made `corpus list`, `corpus inspect`,
and `compare pgvector` surface CLI profile names directly.

That forced the operator to remember that:

- `ec_hnsw` AM maps to `--profile ec_hnsw`
- `ec_diskann` AM maps to `--profile ec_diskann`

This packet makes the storage surface consistent with the rest of the
CLI by adding a `profile` column to the per-index breakdown.

## What changed

### `crates/ecaz-cli/src/commands/bench/storage.rs`

- Expanded the index table header from five columns to six by replacing
  the terse `am` heading with `access method` and inserting a new
  `profile` column next to it.

```rust
idx.set_header(vec![
    "index",
    "access method",
    "profile",
    "reloptions",
    "size",
    "per row",
]);
```

- Added `profile_label_for_access_method(access_method)` using
  `profiles::resolve_by_access_method` so each index row shows the
  matching CLI profile name, or `<unknown>` for AMs the CLI does not
  recognize.

```rust
idx.add_row(vec![
    Cell::new(name),
    Cell::new(&am),
    Cell::new(profile_label_for_access_method(&am)),
    Cell::new(opts),
    Cell::new(format_bytes(size)),
    Cell::new(format!("{:.1} B", per_row_bytes(size, rows))),
]);
```

- Added two unit tests pinning the known and unknown cases:
  - `profile_label_for_access_method_maps_known_profiles`
  - `profile_label_for_access_method_marks_unknown_access_methods`

## Why this slice

- It closes the last obvious raw-AM-only inventory surface in the CLI:
  list, inspect, compare, and now storage all speak in terms of the
  operator's actual `--profile` names.
- It keeps the scope tight: one file, no new SQL, no new shared helpers,
  and no overlap with the `scripts/` deletion lane.
- It improves archived storage tables the same way 11061/11062 improved
  inspect and compare output: readers can tell immediately which profile
  each index belongs to without reverse-mapping AM names in their head.

## Test evidence

```
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 200 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also ran locally for this slice:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Both passed on `pg18`.

## Follow-ups intentionally not in this packet

- Adding a header-level profile summary row to storage output. The
  per-index profile column already answers the operator question without
  widening the table header.
- Deduplicating the AM→profile label helper across `inspect` and
  `storage`. The duplication is small and local; extracting shared
  display helpers would be a separate cleanup slice.
- Reordering or grouping indexes by profile instead of size. The current
  size-descending order remains the most useful view for storage work.
