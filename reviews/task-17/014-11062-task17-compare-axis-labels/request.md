# Review Request: Make `compare pgvector` axis-aware for DiskANN profiles

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/commands/compare/pgvector.rs`

## What this packet is

`ecaz compare pgvector` still spoke HNSW on the ecaz side even after
task-17 added DiskANN as a first-class profile:

- the ecaz tuning flag was named `--ecaz-ef-search`, which is wrong for
  `ec_diskann` because the real scan-time knob is `list_size`
- the output table labeled the ecaz row as plain `ecaz`, which hid both
  the selected profile (`ec_hnsw` vs `ec_diskann`) and the tuning value

This packet fixes that by making the compare surface axis-aware while
keeping the old flag spelling as a compatibility alias.

## What changed

### `crates/ecaz-cli/src/commands/compare/pgvector.rs`

- Renamed the ecaz-side tuning flag to the profile-neutral
  `--ecaz-sweep` and kept `--ecaz-ef-search` as an accepted alias so
  existing invocations do not break.

```rust
/// Ecaz-side tuning value for the selected profile's sweep axis
/// (`ef_search` for HNSW, `list_size` for DiskANN).
#[arg(long = "ecaz-sweep", alias = "ecaz-ef-search", default_value_t = 100)]
pub ecaz_sweep: i32,
```

- Updated the compare run path to set the selected profile's tuning GUC
  from `args.ecaz_sweep` rather than an HNSW-named field.

```rust
client
    .batch_execute(&format!("SET {ecaz_guc} = {}", args.ecaz_sweep))
    .await
    .wrap_err_with(|| format!("SET {ecaz_guc}"))?;
```

- Made comparison table row labels self-describing by embedding the
  engine/profile name plus the active tuning value:
  - `ec_hnsw[ef_search=100]`
  - `ec_diskann[list_size=200]`
  - `pgvector[ef_search=100]`

```rust
ComparisonRow::new(
    &configured_engine_label(profile.name, profile.sweep_axis_label(), args.ecaz_sweep),
    ecaz_recall,
    ecaz_ndcg,
    ecaz_stats,
)
```

- Added three unit tests:
  - `configured_engine_label_is_self_describing`
  - `pgvector_args_accept_generic_ecaz_sweep_flag`
  - `pgvector_args_keep_legacy_ecaz_ef_search_alias`

## Why this slice

- It fixes a concrete DiskANN UX bug without widening scope beyond one
  file in `crates/ecaz-cli/`.
- It makes archived/shared comparison tables readable after the fact:
  the operator can now see which ecaz profile was measured and at what
  sweep value, instead of reverse-engineering it from the shell history.
- It aligns `compare pgvector` with the earlier task-17 bench work
  (default sweeps, sweep-axis labels, AM preflight) so the profile-aware
  vocabulary is consistent across command surfaces.

## Test evidence

```
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 198 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also ran locally for this slice:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Both passed on `pg18`.

## Follow-ups intentionally not in this packet

- Removing the legacy `--ecaz-ef-search` alias entirely. This packet
  keeps old invocations working; if we later choose to drop it, that
  should be a deliberate breaking-change slice with docs/help cleanup.
- Renaming `profile.ef_search_guc` in the registry to a more generic
  field name. That touches bench + compare and is larger than this one
  command UX fix.
- Adding a machine-readable compare output mode (`--json` / `--csv`)
  that carries the same profile+tuning metadata. No such sink exists
  yet.
