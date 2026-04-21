# Review Request: Bench sweep diagnostics use profile axis labels

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/commands/bench/mod.rs`
- `crates/ecaz-cli/src/commands/bench/recall.rs`
- `crates/ecaz-cli/src/commands/bench/latency.rs`
- `crates/ecaz-cli/src/commands/bench/overhead.rs`

## What this packet is

Task-17 already made the bench result tables use the selected profile's
axis name (`ef_search` for HNSW, `list_size` for DiskANN), but the
runtime diagnostics still leaked lower-level GUC vocabulary:

- progress bars showed full GUC names like `ec_diskann.list_size=200`
- the default-sweep banner said only "using profile default [...]"
- `bench recall` still had one `ef_search`-specific error/doc string

This packet makes those diagnostics speak the same operator-facing
language as the tables.

## What changed

### `crates/ecaz-cli/src/commands/bench/mod.rs`

- Added `sweep_value_label(profile, value) -> String`, a small shared
  formatter for progress-bar labels:

```rust
pub(crate) fn sweep_value_label(profile: &IndexProfile, value: i32) -> String {
    format!("{}={value}", profile.sweep_axis_label())
}
```

- Added a unit test pinning both current profiles:
  - `ec_hnsw` → `ef_search=100`
  - `ec_diskann` → `list_size=200`

### `crates/ecaz-cli/src/commands/bench/{recall,latency,overhead}.rs`

- Updated the `--sweep` help strings from "tuning GUC" to "tuning axis".
- Updated the "no `--sweep` provided" banner to name the axis explicitly:

```rust
eprintln!(
    "[recall] no --sweep provided; using profile default {} values {:?}",
    profile.sweep_axis_label(),
    profile.default_sweep
);
```

- Switched progress-bar labels to the shared axis formatter:

```rust
bar.set_message(super::sweep_value_label(profile, *value));
```

That means operators now see `list_size=200` / `ef_search=100` instead
of raw fully-qualified GUC names.

- In `bench recall`, replaced the last `ef_search`-specific error/doc
  wording with the generic "tuning GUC to sweep" phrasing so the file no
  longer reads as HNSW-only.

## Why this slice

- It finishes the bench UX thread started by packets 11055 and 11057:
  default sweeps and table headers already knew the profile axis; now
  the runtime messages do too.
- It helps DiskANN most because `list_size` is the new knob operators
  are learning. Seeing `list_size=...` consistently across stderr and
  table output reduces translation overhead.
- Scope stays tight: one tiny shared helper, three call sites, no
  benchmark logic changes.

## Test evidence

```
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 201 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also ran locally for this slice:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Both passed on `pg18`.

## Follow-ups intentionally not in this packet

- Renaming `profiles::ef_search_guc` to a more generic registry field
  name. That would touch more code than this formatting-only slice.
- Reworking the bench commands to print the axis label in `--help` as a
  dynamic Clap argument name. Clap's static help surface makes that a
  larger change.
- Changing `SET {guc} = {value}` error contexts to hide the raw GUC
  name. For failure debugging, the exact Postgres GUC remains useful.
