# Review Request: Bench commands preflight for the profile access method

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/psql.rs`
- `crates/ecaz-cli/src/commands/bench/mod.rs`
- `crates/ecaz-cli/src/commands/bench/recall.rs`
- `crates/ecaz-cli/src/commands/bench/latency.rs`
- `crates/ecaz-cli/src/commands/bench/overhead.rs`

## What this packet is

`ecaz bench recall --profile ec_diskann` could silently measure the wrong
thing when `<prefix>_corpus` only had HNSW indexes: the KNN SQL still ran,
Postgres fell through to another path, and the CLI reported recall /
latency / overhead as if it had exercised DiskANN.

This packet adds a bench-time preflight that refuses to run unless
`<prefix>_corpus` has at least one index built with the selected
profile's `access_method`. The operator now gets a direct hint to run
`ecaz corpus load --profile <name> ...` first instead of a misleading
table.

## What changed

### `crates/ecaz-cli/src/psql.rs`

- New helper `index_count_with_am(client, table, am) -> Result<i64>`.
- The SQL is the same catalog shape `corpus inspect` already uses for
  listing indexes, reduced to a `count(*)`:

```rust
SELECT count(*)
FROM pg_class t
JOIN pg_index ix ON ix.indrelid = t.oid
JOIN pg_class i  ON i.oid = ix.indexrelid
JOIN pg_am    pam ON pam.oid = i.relam
WHERE t.relname = $1
  AND pam.amname = $2
```

- This keeps the AM check in one place instead of duplicating the
  `pg_class` / `pg_index` / `pg_am` join in three commands.

### `crates/ecaz-cli/src/commands/bench/mod.rs`

- New pure formatter `missing_am_error(profile, am) -> String`:

```rust
format!(
    "no {am} index found for profile {:?}; build one first with `ecaz corpus load --profile {} ...`",
    profile.name, profile.name
)
```

- Two unit tests pin the operator-facing wording:
  - DiskANN happy path (`ec_diskann`)
  - explicit `am` passthrough so the helper does not silently ignore its
    second argument

### `crates/ecaz-cli/src/commands/bench/{recall,latency,overhead}.rs`

- Each command now checks the corpus table immediately after connecting
  and before any sweep work:

```rust
if psql::index_count_with_am(&client, &corpus_table, profile.access_method).await? == 0 {
    return Err(eyre!(
        "{} on {:?}",
        super::missing_am_error(profile, profile.access_method),
        corpus_table
    ));
}
```

- Call-site placement:
  - `recall`: before fetching corpus / queries and before brute-force
    ground truth
  - `latency`: before loading query vectors into memory
  - `overhead`: before EXPLAIN prep and before query sampling

That keeps the failure cheap and early. If the operator asked for
DiskANN but only loaded HNSW indexes, the command now stops before any
measurement or progress bars start.

## Why this slice

- Fixes a concrete DiskANN UX bug without touching surrounding planner,
  SQL-shape, or table-rendering code.
- Reuses the existing catalog join shape from `corpus inspect` rather
  than inventing a second source of truth for AM lookups.
- Keeps scope tight: one SQL helper, one shared error formatter, three
  call sites, no `scripts/` overlap.

## Test evidence

```text
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 185 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also ran local `pg18` verification outside the packet snippet:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

## Follow-ups intentionally not in this packet

- Live integration coverage that exercises the exact failure mode
  (`--profile ec_diskann` against a corpus with only HNSW indexes).
  This packet keeps the test seam pure and local by pinning the message
  formatter.
- Applying the same AM preflight to other operator surfaces that may
  dispatch KNN SQL in the future. Today the bug is specifically in the
  three `ecaz bench` commands.
- Distinguishing "corpus table missing" from "corpus exists but has zero
  indexes for this AM" with separate copy. The requested fix only needs
  the AM-mismatch guard.
