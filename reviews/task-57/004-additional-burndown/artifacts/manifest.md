# Task 57 Packet 004 — Artifact Manifest

## Provenance

- Branch: `task-57`
- Pre-slice HEAD: `aff478ef0` (Task 57/003 closeout draft)
- Post-slice HEAD: this packet's owning commit
- Scope: `src/am/ec_ivf/{scan,vacuum}.rs` only.

## Artifacts

### `block-counts.txt`

Per-file `unsafe { … }` block counts captured by:

```
for f in src/am/ec_ivf/*.rs; do
  echo "$f: $(grep -c 'unsafe {' "$f")"
done
echo "src/ total: $(grep -rh 'unsafe {' src/ | wc -l)"
```

Captured immediately after the final edit in this slice, prior to
commit.

### `cargo-check.log`

Output of `cargo check --no-default-features --features pg18 --lib`.
Compiles cleanly; only pre-existing C-shim `-Wunused-parameter`
warnings remain (`pgstat_internal.h`, `ilist.h`), which are unchanged
since main `9afb2c6b8`.

### `cargo-check-all-targets.log`

Output of `cargo check --all-targets --no-default-features --features pg18`.
Compiles cleanly with the `pg_test` cfg active (exercising all
debug-helper lifts via `src/tests/ec_ivf.rs::ec_ivf_debug!` macro
call sites).

## Not included

Bench gate (recall + latency) is the responsibility of packet 005
closeout, which establishes the Task-57 M5 IVF baseline and runs the
post-slice profile against it.
