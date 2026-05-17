## Feedback: ADR-030 v2 Guarded Flush Output

Read `grouped_v2_flush_output` in `src/am/build.rs`.

### What's right

- Sets metadata to v2 with `TransformKind::Srht`, `SearchCodecKind::GroupedPq`, and
  `RerankCodecKind::ScalarQuantized`. That's the composed pipeline encoded in one
  structured value.
- The flush output is guarded: it only produces v2 metadata if the plan is a v2 plan.
  Scalar builds cannot accidentally emit v2 metadata.

### Concerns

1. **Scalar guard is negative-only.** If a new codec variant is added in the future,
   the flush code may silently default to scalar-v1 metadata. Consider making the
   dispatch exhaustive on `GraphStorageFormat` so a new variant is a compile error
   rather than a silent fallback. (Low priority but easy to do now.)

2. **Flush output and metadata contract.** The metadata contract in packet 312 owns
   the field layout; the flush output must populate every v2 field. If any field is
   missed (e.g. `search_subvector_dim`), the index would be built with default-zero
   fields and scan-side validation in packet 323 may or may not catch it. Worth a
   check that every non-default v2 metadata field is explicitly set by the flush
   output, and that `validate_runtime_scan_format` rejects indexes where any required
   field is zero.

### Observation

This is the packet where v2 metadata actually becomes durable on disk. Before the
build gate is loosened, there should be an explicit test that a v2-built index round-
trips its metadata correctly across a postmaster restart (close index relation, reopen,
re-read metadata). Today this is implicitly covered by pg_tests but not explicitly
named.
