# Review Request: C1 EcVector Extension Upgrade Path

Current head at execution: `913c907`

## Context

While setting up a stable cached source-build measurement surface on the
long-lived pg17 scratch cluster, I hit a real upgrade bug:

- the branch install produced `tqvector--0.1.1.sql`
- but existing databases still on extension version `0.1.0` had no SQL update
  path to pick up the new `ecvector` type and related functions/operators
- as a result, source-build fixture reset failed immediately at
  `embedding ecvector`

This was not just a local measurement issue. Any existing database with
`tqvector` already installed would miss the `ecvector` SQL objects.

## What changed

1. Bumped [tqvector.control](/home/peter/dev/tqvector/tqvector.control) default
   version from `0.1.0` to `0.1.1`.
2. Added [tqvector--0.1.0--0.1.1.sql](/home/peter/dev/tqvector/tqvector--0.1.0--0.1.1.sql),
   which backfills the missing `ecvector` SQL surface for existing databases:
   - `ecvector` type
   - I/O / send / recv functions
   - casts
   - `encode_to_ecvector`
   - inner-product/query operators and operator class

## Why this matters

- new installs already got the full bootstrap path; old installs did not
- the source-backed build and ecvector-adjacent task16 follow-ons now depend on
  those SQL objects existing in upgraded databases, not just fresh installs
- without this, stable cached measurement surfaces on an existing scratch DB
  are brittle or impossible

## Validation

Green checkpoint validation:

```bash
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Operational verification on the pg17 scratch cluster after install:

- copied the new upgrade SQL into the extension directory because
  `cargo pgrx install` only placed `tqvector--0.1.1.sql`, not the
  `0.1.0--0.1.1` transition script
- `ALTER EXTENSION tqvector UPDATE TO '0.1.1'` succeeded
- `pg_type` then showed `ecvector`

## Review focus

1. Is `0.1.1` the right narrow version bump for backfilling `ecvector` into
   existing databases?
2. Do you want a follow-up that ensures the transition script is copied into the
   installed extension dir automatically by the repo workflow, or is the SQL
   upgrade path itself the important merge blocker to fix here?
