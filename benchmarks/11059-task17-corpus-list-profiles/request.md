# Review Request: `ecaz corpus list` shows loaded access methods and profiles

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/profiles.rs`
- `crates/ecaz-cli/src/commands/corpus/list.rs`

## What this packet is

Packet 11058 stopped `ecaz bench {recall,latency,overhead}` from silently
measuring the wrong AM, but it still left an operator-discovery gap:
`ecaz corpus list` only showed a raw index count. You could see that a
corpus had "2 indexes" without knowing whether those were `ec_hnsw`,
`ec_diskann`, a mix of both, or something unrelated.

This packet turns `corpus list` into the quick preflight surface for
"what can I bench on this corpus?" by adding two glance-level columns:

- distinct index access methods on `<prefix>_corpus`
- resolved CLI profile names for those AMs

That makes DiskANN readiness visible before the operator reaches the
bench-time preflight error.

## What changed

### `crates/ecaz-cli/src/profiles.rs`

- New helper `resolve_by_access_method(access_method) -> Option<&IndexProfile>`.
- This is the inverse of the existing `resolve(name)` lookup: commands
  that start from catalog AM names can now recover the CLI profile
  identifier instead of assuming `profile.name == access_method`
  forever.
- Test added:

```rust
assert_eq!(
    resolve_by_access_method("ec_diskann").map(|p| p.name),
    Some("ec_diskann")
);
assert!(resolve_by_access_method("btree").is_none());
```

### `crates/ecaz-cli/src/commands/corpus/list.rs`

- Expanded the SQL from "count indexes" to "count indexes + aggregate
  distinct AM names":

```rust
COUNT(ix.indexrelid)::bigint AS n_indexes,
COALESCE(
    array_agg(DISTINCT am.amname ORDER BY am.amname)
        FILTER (WHERE am.amname IS NOT NULL),
    '{}'::text[]
) AS access_methods
```

- The query uses the same `pg_index` + `pg_class` + `pg_am` join shape
  as `corpus inspect`, but grouped per corpus prefix rather than listed
  per index.
- New table headers:
  - `access methods`
  - `profiles`
- New pure helpers:
  - `profile_names_for_access_methods(&[String]) -> Vec<&'static str>`
  - `format_name_list(&[T]) -> String`
- Output behavior:
  - no indexes: `access methods = <none>`, `profiles = <none>`
  - known AMs: names sorted and deduplicated (`ec_diskann, ec_hnsw`)
  - unknown AMs: still shown in `access methods`, omitted from
    `profiles` so the CLI does not lie about bench support

Example row shape after this packet:

```text
| dbpedia_10k | 10000 | yes | 2 | ec_diskann, ec_hnsw | ec_diskann, ec_hnsw |
```

## Why this slice

- Directly follows 11058: once bench commands fail fast on missing AMs,
  `corpus list` should expose the information operators need to avoid
  that failure in the first place.
- Stays inside `crates/ecaz-cli/`; no overlap with deprecated `scripts/`
  surfaces.
- Keeps `corpus list` lightweight. `inspect` remains the detailed view;
  `list` is now the fast roster of which corpora support which profiles.

## Test evidence

```text
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 190 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also ran local `pg18` verification outside the packet snippet:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

## Follow-ups intentionally not in this packet

- Converting `corpus list` into a full index inventory. `corpus inspect`
  already owns per-index reloptions and sizes; duplicating that here
  would collapse the distinction between the two commands.
- Adding profile coverage warnings ("queries missing", "bench not ready")
  beyond the raw columns. This slice is visibility only.
- Surfacing non-profile AMs as synthetic pseudo-profiles. Unknown AMs are
  intentionally left visible only in the `access methods` column.
