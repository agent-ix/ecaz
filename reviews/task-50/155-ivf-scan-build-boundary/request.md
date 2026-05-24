# Review Request: IVF Scan/Build Pointer Boundaries

## Summary

This checkpoint addresses the soundness-audit finding for remaining IVF helpers that accepted PostgreSQL raw relation and scan pointers through safe APIs.

Code commit: `6e20b334c054631b8a0536ec6f371aa7f50d6d9d`

The reviewer was correct: these helpers depend on live AM callback descriptors, relation descriptors, and debug scan descriptors. The slice makes those preconditions explicit with `unsafe fn` boundaries and caller-side acknowledgments.

## Scope

- Marked IVF empty-bootstrap relation locking unsafe.
- Marked IVF build flush/data-page writer helpers unsafe.
- Marked heap tuple descriptor copying unsafe where it consumes raw heap relations.
- Marked IVF scan descriptor, opaque, heap relation, snapshot, heap OID, and heap-rerank prefetch helpers unsafe.
- Marked IVF pg_test/debug scan wrappers unsafe where they operate on raw scan descriptors or relation pointers.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed; existing unused-import warning remains in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-block-count`: passed; count increase is expected for this explicit-boundary pass.

See `artifacts/manifest.md` for packet-local command provenance and key output lines.
