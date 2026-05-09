# Review Request: README Build Setup

## Summary

The top-level README now gives a prominent, repeatable build path for a
pgrx-managed PostgreSQL 18 setup and points to a fuller source-build guide.

Code checkpoint: `e50ca09c` (`Document repeatable source build setup`)

## Scope

- Changes the README quick start from implicit `cargo pgrx init` plus
  `cargo pgrx install --sudo` to an explicit pgrx-managed PG18 path:
  `cargo pgrx init --pg18 download` and `cargo pgrx run --release pg18`.
- Adds `docs/build-from-source.md` with native prerequisites, pgrx setup,
  existing-PostgreSQL install guidance, operator CLI setup, validation commands,
  PG17 compatibility notes, and troubleshooting.
- Updates Getting Started and Contributing so their setup references align with
  the PG18-default build path.

## Validation

- `git diff --check`
- `git diff --cached --check`

No code tests were run. This is a documentation-only checkpoint.
