# Task 41 invariant #2 review request: DSM and fixed-payload scoped borrows

## Scope

This packet covers code commit `705b2c940e06b40297b2973e74430e2851f3ff5a`
(`Scope invariant2 DSM and UUID helper borrows`).

The slice handles packet 147 seq 2 items G and H, plus the read-only HRTB part
of item C that does not depend on the writable page-helper split:

- HNSW concurrent DSM leader initialization now fills neighbor slots, codes,
  and sources through scoped `with_concurrent_dsm_*_init` callbacks instead of
  open-coded `from_raw_parts_mut` calls.
- HNSW concurrent DSM runtime code/source/neighbor-slot callbacks now use
  higher-ranked callback bounds.
- HNSW shared page tuple, HNSW scan_debug page tuple, and IVF page tuple
  read-only helpers now use higher-ranked callback bounds.
- SPIRE UUID source identity payload reads now go through
  `with_uuid_payload_bytes`, which copies into a fixed-size array and exposes
  it only through a scoped HRTB callback.

## Deferred Items

This packet intentionally does not handle:

- packet 147 seq 2 A/B: writable page-helper read/write split and helper
  consolidation;
- the writable-helper-dependent remainder of C/D;
- F: making `scan_debug` delegate to `shared::with_page_line_tuple_bytes`.

Those are coupled to the HNSW/DiskANN writable page-helper API shape and should
land as a separate local slice.

## Validation

See `artifacts/manifest.md` for command metadata.

- `cargo fmt --all --check` passed with the repository's existing stable-rust
  rustfmt configuration warnings.
- `cargo check --no-default-features --features pg18` passed with the known
  pre-existing unused imports warning in `src/am/mod.rs`.
- `git diff --check 705b2c94^ 705b2c94` passed.

## Reviewer Focus

- Confirm the DSM init `from_raw_parts_mut` sites named in packet 147 seq 2
  item G are now helper-internal.
- Confirm the UUID payload helper prevents a raw slice from escaping the fixed
  payload copy.
- Confirm the read-only HRTB additions do not alter page/DSM behavior.
