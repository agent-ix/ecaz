# Review Request: Per-profile `default_sweep` for bench commands

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/profiles.rs`
- `crates/ecaz-cli/src/commands/bench/recall.rs`
- `crates/ecaz-cli/src/commands/bench/latency.rs`
- `crates/ecaz-cli/src/commands/bench/overhead.rs`

## What this packet is

Makes `--sweep` optional on `ecaz bench {recall,latency,overhead}` by
giving each `IndexProfile` a `default_sweep` array that the CLI falls
back to when the operator supplies no values. Before this change, every
bench invocation required the operator to memorize per-AM tuning ranges
(HNSW `ef_search` ≈ 40–200; DiskANN `list_size` ≈ 64–800). After, a
bare `ecaz bench recall --prefix X --profile ec_diskann` walks a
sensible default sweep and the CLI reports which values it used.

DiskANN benefits most: `list_size` is a new knob with no existing
operator muscle memory, and the default sweep gets them to a first
recall/latency table without a docs lookup.

## What changed

### `crates/ecaz-cli/src/profiles.rs`

- New field `IndexProfile::default_sweep: &'static [i32]` with a doc
  comment that ties it to the three `bench` commands.
- Values:
  - `EC_HNSW.default_sweep = [40, 64, 100, 128, 160, 200]` — matches
    the `--ef-search` defaults documented in
    `docs/RECALL_REAL_CORPUS.md` for the NFR-001 lane.
  - `EC_DISKANN.default_sweep = [64, 128, 200, 400, 800]` — five
    geometric-ish values covering recall floor through saturation
    for the current DiskANN implementation; the top value (800) is
    inside `ECDISKANN_MAX_SCAN_LIST_SIZE` by construction.
- Two new registry-level tests:
  - `every_profile_has_nonempty_default_sweep` — future AM profiles
    can't skip the field without explicit opt-out + CLI update.
  - `default_sweep_is_strictly_ascending` — comfy-table rows print
    in sweep order; unsorted defaults would obscure the recall /
    latency knee.

### `crates/ecaz-cli/src/commands/bench/recall.rs` (plus symmetric changes in `latency.rs` and `overhead.rs`)

Replaces the old

```rust
if args.sweep.is_empty() {
    return Err(eyre!("--sweep requires at least one value ..."));
}
```

with

```rust
let sweep_values: Vec<i32> = if args.sweep.is_empty() {
    if profile.default_sweep.is_empty() {
        return Err(eyre!(
            "--sweep is required for profile {:?} (no default sweep registered)",
            profile.name
        ));
    }
    eprintln!(
        "[recall] no --sweep provided; using profile default {:?}",
        profile.default_sweep
    );
    profile.default_sweep.to_vec()
} else {
    args.sweep.clone()
};
```

…and updates the iteration loop from `for value in &args.sweep` to
`for value in &sweep_values`. The stderr banner (`[recall]` / `[latency]` /
`[overhead]`) mirrors the command's existing diagnostic prefix so
operators can grep runs by command.

The error path is preserved for a future AM whose profile intentionally
ships `default_sweep: &[]` (e.g., `SweepAxis::None`). In that case the
operator still gets a clear message pointing at the profile name.

## Why this slice

- Wholly inside `crates/ecaz-cli/`; no `scripts/*` overlap with the
  deletion lane on `main`.
- Improves DiskANN UX where the user's first interaction with the
  tool is "I have no idea what values are reasonable for
  `list_size`". The sweep is now printed right in stderr so the
  operator learns the range without leaving the CLI.
- Near-zero cost: ~15 lines of arithmetic per command, no new DB
  interaction. Unit-testable via the profile fields alone.

## Test evidence

```
$ cargo test -p ecaz-cli 2>&1 | tail -3
test result: ok. 177 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
finished in 0.01s
```

Up from 175 in packet 11054. The two new tests guard the registry
invariant (nonempty + ascending) so profile additions can't silently
regress either property.

## Follow-ups intentionally not in this packet

- Splitting `default_sweep` into `default_sweep_recall` and
  `default_sweep_latency` if those ever diverge. Today they coincide —
  recall, latency, and overhead all want the same frontier walk — so
  one field keeps the registry readable.
- Reading `default_sweep` values from a TOML / env override. No
  operator has asked for it yet; the compile-in default is one line
  to change.
- Exposing `default_sweep` in `ecaz --help` output per command. Clap's
  existing value-parser path would need a runtime-resolved default,
  which is more infrastructure than one packet can justify against a
  stderr banner that already prints the values on use.
