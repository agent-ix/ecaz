# Remote Search Test Split Strategy

Code checkpoints:

- `23a751e497835c62aa998fa7a4b5db0e8b9b5631`
- `f386b5f900286489e3ae603d17990babfda27362`

## Shrink List

- `src/tests/remote_search.rs`: remove as a monolithic destination file.
- `src/tests/mod.rs`: keep line count flat; only retarget the include.

No new test body should be added to either shrink-list file. New remote-search
test work should land under `src/tests/remote_search/`.

## Analysis

`src/tests/remote_search.rs` had 12,246 lines at `HEAD~2`. It was included
inside the existing pg_test module from `src/tests/mod.rs`, so the low-risk
split is to preserve the include-based compilation model rather than move tests
into Rust submodules that might alter helper visibility or pg_test discovery.

The file is mostly pg_test bodies, with no reusable module-level helper block
at the top. The tests cluster into these concern areas:

- Remote search SQL, coordinator, request, readiness, execution, and final
  contract checks.
- Tuple-payload and heap-resolution checks.
- Coordinator result, remote node descriptor, and catalog checks.
- Production executor summaries and transport/receive fault tests.
- libpq executor behavior.
- Remote epoch manifest and Phase 7 policy checks.
- Catalog cleanup, lifecycle policy, reaper, mode mismatch, and remote PK SELECT
  isolation checks.

## Chosen Split

The first split is intentionally contiguous. It preserves order exactly except
for the relative `include_str!` path fixes required by the extra directory
level, plus removal of separator-only blank lines at chunk EOFs so
`git diff --check` passes.

Created files:

- `src/tests/remote_search/contracts.rs`
- `src/tests/remote_search/tuple_heap.rs`
- `src/tests/remote_search/coordinator_catalog.rs`
- `src/tests/remote_search/production_summary.rs`
- `src/tests/remote_search/transport_faults.rs`
- `src/tests/remote_search/receive_faults.rs`
- `src/tests/remote_search/libpq_executor.rs`
- `src/tests/remote_search/node_catalog.rs`
- `src/tests/remote_search/epoch_manifest.rs`
- `src/tests/remote_search/catalog_cleanup_policy.rs`
- `src/tests/remote_search/mod.rs`

## Next Split Rules

- Do not add test bodies back to `src/tests/remote_search.rs`; it has been
  deleted.
- Do not grow `src/tests/mod.rs`; keep it as an include routing file.
- New remote-search tests should go into the appropriate
  `src/tests/remote_search/*.rs` concern file.
- If a concern file grows above roughly 3,000 lines, split it again before
  adding more tests.
