# Review Request: Show ready CLI profiles in `ecaz corpus inspect`

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/commands/corpus/inspect.rs`

## What this packet is

`ecaz corpus inspect` already listed raw index access methods on
`<prefix>_corpus`, but that still left the operator translating AM names
back into CLI profile names by hand before they could run
`ecaz bench ... --profile ...` or `ecaz compare pgvector --profile ...`.

This packet makes the detailed inspect surface actionable:

- the header now summarizes which benchable CLI profiles are ready for
  the corpus (`bench profiles`)
- each index row gets a `profile` column that maps the raw access method
  back to the matching CLI profile name

That keeps the UX inside `ecaz-cli` and avoids a detour through docs or
source just to answer "which profile should I pass here?"

## What changed

### `crates/ecaz-cli/src/commands/corpus/inspect.rs`

- Added a `bench profiles` row to the header based on the distinct access
  methods present on `<prefix>_corpus`. The row is only considered ready
  when the companion `<prefix>_queries` table exists; otherwise it shows
  `<queries missing>` because bench commands cannot run without queries.

```rust
let access_methods: Vec<String> = rows.iter().map(|r| r.get::<_, String>(1)).collect();
header.add_row(vec![
    "bench profiles".into(),
    Cell::new(bench_ready_profiles_label(
        queries_rows >= 0,
        &access_methods,
    )),
]);
```

- Expanded the index table from four columns to five by inserting a
  `profile` column between `access method` and `reloptions`.

```rust
idx.set_header(vec![
    "index",
    "access method",
    "profile",
    "reloptions",
    "size",
]);
```

- Added two pure helpers:
  - `profile_label_for_access_method(access_method)` maps known AMs via
    `profiles::resolve_by_access_method` and renders `<unknown>` for
    anything the CLI does not recognize.
  - `bench_ready_profiles_label(has_queries, access_methods)` sorts and
    deduplicates known profile names, returns `<none>` when no known AMs
    are present, and returns `<queries missing>` when the queries table
    is absent.
- Added five unit tests covering known/unknown AM mapping plus the three
  bench-profile summary cases (queries missing, sorted+deduped known
  profiles, unknown-only access methods).

## Why this slice

- Complements packet 11059 (`ecaz corpus list`) with the detailed
  per-index view. `corpus list` now answers "which corpora have a
  DiskANN profile anywhere?"; `corpus inspect` answers "which exact
  indexes are on this corpus, and which CLI profile names do they map
  to?"
- Keeps the scope tight: one file, no new DB helpers, no overlap with
  the `scripts/` deletion lane.
- Fits the recent AM-preflight work: after 11058 and 11060 made bench
  and compare reject the wrong index family, this packet gives operators
  a direct CLI way to see the right profile name before they rerun.

## Test evidence

```
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 195 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also ran locally for this slice:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Both passed on `pg18`.

## Follow-ups intentionally not in this packet

- Coloring or otherwise emphasizing the "best" profile in inspect
  output. The command is read-only inventory; policy would add opinion
  beyond this packet's remit.
- Extending the same profile label into any future machine-readable
  `corpus inspect --json` output. No such sink exists yet.
- Reformatting or reordering index rows by profile. The existing
  alphabetical `ORDER BY i.relname` remains stable and predictable.
