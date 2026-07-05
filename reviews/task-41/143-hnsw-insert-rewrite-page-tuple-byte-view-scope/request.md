# Task 41 invariant #2 review request: HNSW insert write tuple byte views

## Scope

This packet covers commit `ee857d12b85d85c1468a50d1497b2351bc24f226`
(`Scope HNSW insert rewrite page tuple byte views`).

The slice keeps HNSW insert page-tuple byte access local to
`src/am/ec_hnsw/insert.rs` by introducing an insert-local
`with_writable_page_tuple_bytes` helper and routing the remaining rewrite
paths through it:

- backlink neighbor rewrites in `add_backlinks_on_page`
- scalar duplicate heap-TID coalescing
- TurboQuant V3 duplicate heap-TID coalescing
- PqFastScan duplicate heap-TID coalescing

The helper owns line-pointer lookup, bounds checking, the short immutable byte
view used for decode, and the exact tuple pointer used for same-size rewrite.
Callers decode from the closure-local byte view and copy encoded bytes back
through the closure-local pointer only after preserving the existing size
checks.

## Reviewer focus

- Confirm the helper keeps raw page tuple views lexically scoped and does not
  extend tuple byte borrows beyond decode/rewrite.
- Confirm the no-op duplicate paths preserve the previous WAL finish/return
  behavior when the heap TID is already present.
- Confirm the backlink retry behavior is unchanged while the tuple rewrite
  surface is narrowed.

## Validation

See `artifacts/manifest.md` for command metadata.

- `cargo fmt --all --check` passed with the repository's existing stable-rust
  rustfmt configuration warnings.
- `cargo check --no-default-features --features pg18` passed with the known
  pre-existing unused imports warning in `src/am/mod.rs`.
- `git diff --check HEAD -- src/am/ec_hnsw/insert.rs` passed.
