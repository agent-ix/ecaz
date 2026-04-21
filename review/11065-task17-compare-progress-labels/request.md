# Review Request: `compare pgvector` progress labels use configured engine names

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/commands/compare/pgvector.rs`

## What this packet is

Packet 11062 made the final comparison table profile-aware by labeling
rows like `ec_diskann[list_size=200]`, but the long-running compare
progress bars and KNN error contexts still used generic labels:

- `ecaz`
- `pgvector`

That meant the table output was self-describing while the live progress
surface was not. This packet closes that gap by reusing the same
configured engine labels everywhere in the compare run.

## What changed

### `crates/ecaz-cli/src/commands/compare/pgvector.rs`

- Hoisted the configured labels into local variables before the two
  engine measurements:

```rust
let ecaz_label =
    configured_engine_label(profile.name, profile.sweep_axis_label(), args.ecaz_sweep);
let pgv_label = configured_engine_label("pgvector", "ef_search", args.pgvector_ef_search);
```

- Passed those labels into `measure_engine(...)` instead of the generic
  `"ecaz"` / `"pgvector"` strings:

```rust
let (ecaz_recall, ecaz_ndcg, ecaz_stats) = measure_engine(
    &client,
    &ecaz_label,
    ...
);
```

That changes:

- progress bars from `[compare ecaz] ...` to
  `[compare ec_diskann[list_size=200]] ...`
- `wrap_err_with(|| format!("{label} KNN"))` contexts to include the same
  configured engine label

- Reused the same two labels when building the final `ComparisonRow`s, so
  the live progress surface and the printed table now agree exactly.

- Updated `comparison_row_carries_engine_label_and_metrics` to pin a
  realistic configured label rather than the old bare `"ecaz"` string.

## Why this slice

- It finishes the 11062 UX change cleanly instead of leaving half the
  compare command generic and half profile-aware.
- It is extremely narrow: one file, no new helpers, no SQL changes, no
  behavioral changes beyond the operator-facing label text.
- It helps on the longest-running compare commands, where the progress
  bar is the surface the operator stares at while the benchmark runs.

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

- Reformatting the top-level `[compare] ...` lifecycle banners
  (`fetching corpus`, `computing ground truth`, etc.) to include the
  configured engine names. Those are stage banners, not per-engine
  progress labels.
- Adding a dedicated helper that returns both compare labels at once.
  Two local variables keep this slice smaller than introducing another
  abstraction.
- Any further compare-table schema changes. This packet is only about
  label consistency between progress output and final rows.
