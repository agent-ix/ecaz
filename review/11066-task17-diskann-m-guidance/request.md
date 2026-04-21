# Review Request: DiskANN `--m` rejection points operators at reloptions

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/commands/corpus/load.rs`

## What this packet is

`ecaz corpus load --profile ec_diskann --m ...` was rejected, but the
error only said to use `--reloption` for AM-specific tunables. That is
technically true, but it still leaves the operator to guess which
DiskANN knobs are valid and what a replacement command should look like.

This packet keeps the behavior the same and makes the guidance
DiskANN-specific: the rejection now names the known DiskANN reloptions
and shows a concrete `ec_diskann` example command.

## What changed

### `crates/ecaz-cli/src/commands/corpus/load.rs`

- Replaced the inline generic `--m` rejection with a dedicated pure
  helper:

```rust
if !profile.sweep_axis_is_m() && !args.m.is_empty() {
    return Err(eyre!(unsupported_m_error(profile)));
}
```

- Added `unsupported_m_error(profile)` so the operator sees a
  DiskANN-oriented hint instead of a generic AM message:

```rust
fn unsupported_m_error(profile: &IndexProfile) -> String {
    format!(
        "--m is not supported by profile {:?}; use --reloption for {} tuning instead (known keys: {}). Example: `ecaz corpus load --profile {} --reloption graph_degree=48 --reloption alpha=1.2 ...`",
        profile.name,
        profile.name,
        profile.known_reloptions.join(", "),
        profile.name
    )
}
```

- Added a unit test that pins the operator-facing guidance for
  `ec_diskann`:

```rust
#[test]
fn unsupported_m_error_points_diskann_operators_at_reloptions() {
    let err = unsupported_m_error(&EC_DISKANN);
    assert!(err.contains("--m is not supported by profile \"ec_diskann\""));
    assert!(err.contains("known keys: graph_degree, build_list_size, list_size"));
    assert!(err.contains("--profile ec_diskann --reloption graph_degree=48"));
}
```

## Why this slice

- It is directly about DiskANN operator UX, not a generic CLI cleanup.
- It fixes a concrete confusion point on the main DiskANN load path:
  `--m` is meaningful for HNSW, but not for `ec_diskann`.
- Scope stays tight to one file, one pure helper, and one test.

## Test evidence

```text
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 202 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also ran locally for this slice on `pg18`:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

## Follow-ups intentionally not in this packet

- Expanding the same style of profile-specific guidance to other invalid
  flag/profile combinations. This packet only fixes the DiskANN `--m`
  path.
- Changing which reloptions `ec_diskann` advertises as known. This
  packet only improves the error copy around the existing profile
  metadata.
- Adding an integration test that shells the full CLI. The requested
  slice stays local by testing the pure formatter.
