# Review Request: Task 28 IVF Training Sample Collection

Scope: Phase 3 heap-scan sample checkpoint. Populated `ec_ivf` builds now
collect and validate heap vectors, select deterministic training samples, and
train centroids before the still-explicit populated-write gate.

Task: `plan/tasks/28-ivf-access-method.md` Phase 3

Branch: `task28-ivf`

Head SHA: `422696d5d29520aa72a2cf5a1e75c728303f5a44`

Owner: coder2

Files:

- `src/am/ec_ivf/build.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo test`
- `cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- `git diff --check`
- `git diff --cached --check`

Final validation highlights:

- `cargo test`: main pg18 lib suite reported 577 passed, 0 failed, 4 ignored;
  proptests, recall smoke, size assertions, and doc tests also passed.
- `cargo pgrx test pg17`: main pg17 lib suite reported 574 passed, 0 failed,
  4 ignored; proptests, recall smoke, size assertions, and doc tests also
  passed.
- Clippy completed cleanly with `-D warnings`.

## Summary

This slice replaces the first-row populated-build error with a real build-side
collection path:

- Resolves the indexed column kind for single-column `ecvector` or `tqvector`
  indexes and keeps expression/partial indexes rejected.
- Rejects NULL indexed values, invalid heap TIDs, non-finite gamma values,
  zero-norm training vectors, and inconsistent dimensions.
- Decodes `ecvector` heap values as f32 source vectors and produces the
  canonical TurboQuant payload for later posting tuples.
- Decodes `tqvector` payloads and derives an approximate f32 vector through
  the existing `ProdQuantizer::decode_approximate` path for centroid training.
- Applies deterministic training-sample selection from the collected heap
  tuples using the Phase 3 trainer helper and configured seed.
- Trains spherical centroids before failing at the populated-write gate with a
  specific count of heap tuples, training samples, and centroids collected.
- Adds unit coverage for auto and explicit sample count behavior,
  deterministic sample collection, dimension rejection, zero-norm rejection,
  and training from the selected sample.

## Review Focus

Please review for:

- Whether deriving approximate training vectors from indexed `tqvector` is the
  right first behavior, or whether `tqvector` IVF builds should require a
  source/rerank column before populated writes are enabled.
- Whether the `training_sample_rows = 0` auto cap of 10,000 rows is acceptable
  for the first build path.
- Whether the build callback should continue to collect all heap tuples before
  sampling, or whether this should become reservoir sampling before large-table
  populated builds are enabled.
- Whether the current type/shape rejection messages are specific enough for
  PostgreSQL users.
- Whether keeping populated page writes gated after centroid training is the
  right checkpoint boundary before directory/posting-list metadata updates.

## Non-Goals

This packet does not write populated IVF centroids or posting-list tuples to
disk, update metadata with trained dimensions/list heads, implement WAL-safe
list-directory updates, implement bulk assignment storage, scan routing, live
insert, vacuum, planner costing, or any measurement claim.
