# Task 41 invariant #2 review request: HNSW vacuum rewrite tuple bytes

## Scope

This packet covers commit `51a5b6b92d44464603766e349d654c02e19b96a8`
(`Scope HNSW vacuum rewrite page tuple bytes`).

The slice keeps HNSW vacuum rewrite byte access local to
`src/am/ec_hnsw/vacuum.rs` by adding a vacuum-local
`with_writable_vacuum_tuple_bytes` helper and routing these rewrite paths
through it:

- pass1 element tuple rewrites in `apply_page_pass1_updates`
- layer repair neighbor tuple rewrites in `apply_repair_plans_on_page`
- pass2 neighbor tuple rewrites in `apply_page_pass2_updates`

The helper owns line-pointer lookup, tuple bounds checking, the short tuple
byte view, and the exact rewrite pointer. The caller closures preserve the
existing same-size encoded tuple checks before copying bytes back.

## Reviewer focus

- Confirm the helper keeps writable page tuple access scoped to the rewrite
  closure.
- Confirm pass1 element updates still cover scalar, TurboQuant V3, and
  PqFastScan element variants.
- Confirm repair paths still skip WAL finish when no page tuple changes are
  made.

## Validation

See `artifacts/manifest.md` for command metadata.

- `cargo fmt --all --check` passed with the repository's existing stable-rust
  rustfmt configuration warnings.
- `cargo check --no-default-features --features pg18` passed with the known
  pre-existing unused imports warning in `src/am/mod.rs`.
- `git diff --check HEAD -- src/am/ec_hnsw/vacuum.rs` passed.
