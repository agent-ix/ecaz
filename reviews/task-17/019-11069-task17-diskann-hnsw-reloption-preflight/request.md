# Review Request: DiskANN load rejects HNSW-only reloptions up front

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/commands/corpus/load.rs`

## What this packet is

`ecaz corpus load --profile ec_diskann --reloption ...` still had one
operator trap left after packets 11066 and 11067: if someone copied HNSW
reloptions such as `m`, `ef_construction`, or `build_source_column` into
the DiskANN load command, the CLI only warned that the keys were unknown
and then let `CREATE INDEX` fail later with generic Postgres copy.

This packet turns that into an explicit DiskANN preflight. HNSW-only
reloptions now fail before any file inspection or DB work starts, with a
message that points the operator at valid DiskANN knobs instead.

## What changed

### `crates/ecaz-cli/src/commands/corpus/load.rs`

- Added a single source of truth for the HNSW-only reloptions that should
  never reach the DiskANN path:

```rust
const HNSW_ONLY_RELOPTIONS: &[&str] = &["m", "ef_construction", "build_source_column"];
```

- `run(...)` now checks those keys before the generic unknown-reloption
  warning path:

```rust
let hnsw_only_reloptions = foreign_hnsw_reloption_keys(profile, &args.reloptions);
if !hnsw_only_reloptions.is_empty() {
    return Err(eyre!(unsupported_hnsw_reloption_error(
        profile,
        &hnsw_only_reloptions
    )));
}
```

- Added two pure helpers:

```rust
fn foreign_hnsw_reloption_keys(
    profile: &IndexProfile,
    reloptions: &[(String, String)],
) -> Vec<String>

fn unsupported_hnsw_reloption_error(profile: &IndexProfile, keys: &[String]) -> String
```

- The error message names the bad HNSW-only keys and shows a DiskANN
  replacement example:

```rust
"profile {:?} does not support HNSW-only reloption{} {}; use DiskANN reloptions instead ..."
```

- Added tests that pin:
  - deduped HNSW-only key detection on `ec_diskann`
  - no false positives on `ec_hnsw`
  - the operator-facing DiskANN error copy

## Why this slice

- It fixes another real DiskANN load-path failure, not generic tool
  polish.
- It keeps the scope tight to one file and a pure preflight seam.
- It completes the family of HNSW-to-DiskANN copy/paste mistakes:
  `--m`, `--ef-construction`, and now HNSW-only `--reloption` keys all
  fail loudly and specifically.

## Test evidence

```text
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 207 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also ran locally for this slice on `pg18`:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

## Follow-ups intentionally not in this packet

- Rejecting every possible unknown reloption with a hard error. This
  packet only tightens the specific HNSW-only keys that mislead DiskANN
  operators today.
- Detecting the inverse case for future non-HNSW profiles beyond
  `ec_diskann`. Today task 17 only needs the DiskANN path.
- Reading AM option metadata from Postgres instead of keeping this list
  in the CLI. A three-key constant is smaller than a new metadata layer
  for this slice.
