# Review Request: README Build Setup

## Summary

The top-level README now gives a prominent, repeatable build path for a
pgrx-managed PostgreSQL 18 setup, a compact compatibility/platform summary,
and links to a fuller source-build guide.

Documentation checkpoints:

- `e50ca09c` (`Document repeatable source build setup`)
- `55d4a1f7` (`Tighten README platform and lifecycle docs`)

## Scope

- Changes the README quick start from implicit `cargo pgrx init` plus
  `cargo pgrx install --sudo` to an explicit pgrx-managed PG18 path:
  `cargo pgrx init --pg18 download` and `cargo pgrx run --release pg18`.
- Adds `docs/build-from-source.md` with native prerequisites, pgrx setup,
  existing-PostgreSQL install guidance, operator CLI setup, validation commands,
  PG17 compatibility notes, and troubleshooting.
- Updates Getting Started and Contributing so their setup references align with
  the PG18-default build path.
- Adds a README compatibility matrix covering PG18/PG17, pgrx, Rust, Linux
  x86_64, macOS Apple Silicon, and the local `target-cpu=native` build policy.
- Adds expected smoke-query output to the README and Getting Started.
- Adds upgrade, reinstall, and uninstall guidance to `docs/build-from-source.md`.
- Replaces the README's detailed per-index SQL blocks with a compact access
  method table that links deeper SQL examples to `docs/usage.md`, leaving room
  for SPIRE docs to land without overloading the top-level README.

## Validation

- `git diff --check`
- `git diff --cached --check`

No code tests were run. This is a documentation-only checkpoint.
