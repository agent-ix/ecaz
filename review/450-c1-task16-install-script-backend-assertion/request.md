# Review Request: C1 Task16 Install-Script Backend Assertion

Current head at execution: `324481a`

## Context

Packet `440` hit a real workflow hazard:

- an initial scratch run produced flat `~26.96ms` numbers
- the root cause was a stale backend module: scratch had not been reinstalled
  on current head yet, so the postmaster was still loading an older
  `tqvector.so`

Task 16's plan had one explicit hygiene item to close that seam:

- make `scripts/install_adr030_pg17_pg_test.sh` assert that the installed
  backend `.so` is actually the just-built artifact

This slice closes that item.

## What changed

Updated:

- [scripts/install_adr030_pg17_pg_test.sh](/home/peter/dev/tqvector/scripts/install_adr030_pg17_pg_test.sh:1)
- [plan/tasks/16-turboquant-iteration.md](/home/peter/dev/tqvector/plan/tasks/16-turboquant-iteration.md:328)

### 1. The install script now fails fast on stale backend copies

After `cargo pgrx install --release --features 'pg17 pg_test' --no-default-features`,
the script now:

1. resolves the installed module path from:

   ```bash
   pg_config --pkglibdir
   ```

2. expects the build artifact at:

   ```bash
   target/release/libtqvector.so
   ```

3. verifies both files exist
4. compares them with `cmp -s`
5. if they differ, prints:
   - built path
   - installed path
   - built/install SHA-256
   - built/install ELF Build ID when available
6. exits non-zero on mismatch

So the script no longer silently succeeds after writing the wrong backend module
or after some future packaging change points install at the wrong artifact.

### 2. Success path is explicit too

On a good install, the script now prints:

- the installed backend path
- the installed backend SHA-256
- a clear `backend .so assertion passed`

That gives the measurement workflow a positive proof point instead of only a
manual "the numbers look sane now" heuristic.

## Important implementation note

`cargo pgrx install` still does its normal SQL-generation `pgrx_embed` rebuild
after copying the release `.so`. The assertion is intentionally anchored to the
installed backend module itself, not to assumptions about the rest of the
install log.

On the real scratch path used for task 16:

```bash
./scripts/install_adr030_pg17_pg_test.sh --pgrx-home /home/peter/.pgrx
```

the assertion passed after install, which means the backend module loaded by the
scratch postmaster is the same bytes as the just-built release artifact.

## Why this matters

This converts packet `440`'s stale-install diagnosis from:

- "we noticed impossible numbers and debugged backwards"

into:

- "the install step itself proves the backend module matches current head"

That is exactly the kind of cheap safety belt task 16 needed before running more
ADR-044 cells.

## Validation

Ran on this exact tree:

```bash
./scripts/install_adr030_pg17_pg_test.sh --pgrx-home /home/peter/.pgrx
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

## Review focus

1. Is `target/release/libtqvector.so` the right build artifact to compare
   against the installed backend module for this script's contract?
2. Is `cmp -s` plus SHA/Build-ID diagnostics the right failure mode, or is a
   weaker metadata-only check hiding in the current pgrx workflow that I
   missed?
3. Does the plan close-out text describe the safety belt precisely enough for
   future task-16 measurement packets to rely on it?
