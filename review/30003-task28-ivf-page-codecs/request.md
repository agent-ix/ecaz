# Review Request: Task 28 IVF Page Codecs

Scope: Phase 2 layout-codec checkpoint. Defines the on-disk codec surface
that later IVF training, build, scan, insert, and vacuum work will consume.

Task: `plan/tasks/28-ivf-access-method.md` Phase 2

Branch: `task28-ivf`

Head SHA: `dd960ea7c1a6d8e4eccb7abf9f088eb83621b42f`

Owner: coder2

Files:

- `src/am/ec_ivf/page.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo test`
- `cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- `git diff --cached --check`

## Summary

This slice establishes the first concrete `ec_ivf` disk-layout codecs:

- Metadata now carries dimensions, training version, centroid head,
  directory head, total live/dead tuple counts, and insert-since-build drift
  state in addition to reloptions already persisted by the empty-index slice.
- Centroids are stored as dedicated data-page tuples with list id,
  dimensions, and finite f32 centroid values.
- List-directory entries have a fixed codec for list id, posting-list
  head/tail block refs, live count, dead count, and insert-since-build count.
- Posting-list tuples use a profile-neutral payload, preserve inline
  duplicate heap TIDs, carry a finite `gamma`, and optionally point to a
  rerank/source tuple through an item pointer.
- `DataPage` and `DataPageChain` now have IVF-specific insert/read helpers
  for centroid, directory, and posting tuples.
- Unit coverage checks metadata roundtrip/truncation, block refs, centroid
  roundtrip/dimension mismatch, directory roundtrip, duplicate heap-TID
  posting payloads, heap-TID overflow rejection, page-chain extension, and
  page-fit helpers.

## Review Focus

Please review for:

- Whether expanding the metadata special area from 64 to 80 bytes is
  acceptable at this pre-release IVF stage.
- Whether metadata should keep one `format_version = 1` after this expansion
  or bump immediately before any populated index can be written.
- Whether centroid tuples should remain one tuple per list or use packed
  centroid pages before training lands.
- Whether the list-directory entry should track block refs, item pointers,
  or both once WAL-safe list-tail updates are implemented.
- Whether the profile-neutral posting payload is enough for TurboQuant,
  PqFastScan, and RaBitQ, or whether any profile needs a distinct hot/cold
  tuple now.
- Whether preserving HNSW's inline duplicate heap-TID capacity is the right
  first duplicate handling contract for IVF.
- Whether `gamma` belongs in every posting tuple for both `tqvector` and
  `ecvector`, or should become format-dependent.

## Non-Goals

This packet does not implement centroid training, heap sample collection,
bulk assignment, WAL-safe directory updates, populated builds, scan scoring,
live insert, vacuum repair, recall validation, or planner costing.
