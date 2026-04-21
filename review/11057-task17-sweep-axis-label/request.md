# Review Request: Bench tables use the profile's sweep-axis name as column header

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/profiles.rs`
- `crates/ecaz-cli/src/commands/bench/recall.rs`
- `crates/ecaz-cli/src/commands/bench/latency.rs`
- `crates/ecaz-cli/src/commands/bench/overhead.rs`

## What this packet is

The three `ecaz bench` commands print a comfy-table whose first column is
the swept value. Until now that column was headed `"sweep"`, which
erases the only piece of information the operator cares about when they
come back to the table a week later: *which knob was I sweeping?*

For HNSW that knob is `ef_search`; for DiskANN it's `list_size`. Those
names were already in the progress-bar message (`[recall ef_search=200]`)
but not in the archived/shared table. This packet closes that gap.

## What changed

### `crates/ecaz-cli/src/profiles.rs`

- New method `IndexProfile::sweep_axis_label() -> &'static str` that
  derives the short label from `ef_search_guc`:
  - `ec_hnsw.ef_search` → `ef_search`
  - `ec_diskann.list_size` → `list_size`
  - `None` (future `SweepAxis::None` profiles) → `"sweep"` fallback.
- Test: `sweep_axis_label_strips_module_prefix` pinning both shapes.

### `crates/ecaz-cli/src/commands/bench/{recall,latency,overhead}.rs`

- Replaced the literal `"sweep"` header with
  `profile.sweep_axis_label()`. No other columns changed.
- No change to the progress-bar messages — those still say
  `{guc}={value}`, which now reinforces the column header rather than
  duplicating it.

## Why this slice

- Tiny, self-contained, three identical edits plus one method.
- Improves DiskANN operator muscle memory ("list_size=200 gave me
  recall 0.92") without a docs lookup.
- Symmetric with packet 11055 (default_sweep): the CLI is gradually
  pulling every profile-specific knob name into the UX instead of
  hiding them behind generic vocabulary.
- All inside `crates/ecaz-cli/`; no overlap with the script-deletion
  lane on `main`.

## Test evidence

```
$ cargo test -p ecaz-cli 2>&1 | tail -3
test result: ok. 183 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
finished in 0.01s
```

Up from 182 in packet 11056 (+1 for `sweep_axis_label_strips_module_prefix`).

## Follow-ups intentionally not in this packet

- Piping the same label into the CSV/JSON output once we add one.
  Today the stdout table is the only sink.
- Pretty-printing the GUC source alongside the column header
  (e.g. `list_size (ec_diskann.list_size)`). Adds width for little
  gain; reachable from `--help` or the progress bar.
