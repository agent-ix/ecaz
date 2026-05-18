# Review Request: DiskANN load rejects HNSW-only `--ef-construction`

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/commands/corpus/load.rs`

## What this packet is

`ecaz corpus load --profile ec_diskann --ef-construction ...` did not do
what the operator asked for: the flag is HNSW-only, but the DiskANN path
silently ignored it and continued building the index.

This packet turns that into an explicit DiskANN-facing error instead of a
silent no-op. It also preserves the existing HNSW default of
`ef_construction=128` when the flag is not passed.

## What changed

### `crates/ecaz-cli/src/commands/corpus/load.rs`

- `LoadArgs.ef_construction` now records whether the operator passed the
  flag at all:

```rust
#[arg(long)]
pub ef_construction: Option<i32>,
```

- `run(...)` now rejects the HNSW-only flag on non-HNSW profiles before
  any file inspection or DB work starts:

```rust
if !profile.sweep_axis_is_m() && args.ef_construction.is_some() {
    return Err(eyre!(unsupported_ef_construction_error(profile)));
}
```

- HNSW still gets the same default build setting via a local constant:

```rust
const DEFAULT_HNSW_EF_CONSTRUCTION: i32 = 128;

args.ef_construction.unwrap_or(DEFAULT_HNSW_EF_CONSTRUCTION)
```

- Added a pure formatter that points DiskANN operators at valid reloptions
  and a concrete replacement command:

```rust
fn unsupported_ef_construction_error(profile: &IndexProfile) -> String {
    format!(
        "--ef-construction is not supported by profile {:?}; use --reloption for {} tuning instead (known keys: {}). Example: `ecaz corpus load --profile {} --reloption graph_degree=48 --reloption build_list_size=128 ...`",
        profile.name,
        profile.name,
        profile.known_reloptions.join(", "),
        profile.name
    )
}
```

- Added `unsupported_ef_construction_error_points_diskann_operators_at_reloptions`
  to pin the operator-facing copy for `ec_diskann`.

## Why this slice

- It fixes a real DiskANN behavior bug, not a label-only cleanup:
  previously the operator could pass a tuning flag that did nothing.
- It stays inside the DiskANN load path in one file.
- It pairs naturally with packet 11066: both HNSW-only load flags now
  fail loudly on `ec_diskann` instead of misleading the operator.

## Test evidence

```text
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 203 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also ran locally for this slice on `pg18`:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

## Follow-ups intentionally not in this packet

- Reworking other `corpus load` arguments to preserve explicit-vs-default
  provenance. This packet only needs that behavior for `--ef-construction`.
- Adding full CLI integration coverage for the error path. The requested
  slice stays local by testing the pure formatter and preserving the
  existing HNSW plan tests.
- Expanding the rejection to non-DiskANN profiles that may appear later.
  Today the active bug is the `ec_diskann` path.
