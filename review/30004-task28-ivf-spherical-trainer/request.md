# Review Request: Task 28 IVF Spherical Trainer

Scope: Phase 3 pure-training checkpoint. Adds deterministic spherical
k-means helpers for the `ec_ivf` router without wiring populated builds yet.

Task: `plan/tasks/28-ivf-access-method.md` Phase 3

Branch: `task28-ivf`

Head SHA: `8ebabcf2ac7e2bb9ccdbdd3cbc975889966a9f33`

Owner: coder2

Files:

- `src/am/ec_ivf/training.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo test`
- `cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- `git diff --cached --check`

## Summary

This slice adds the pure training layer for the IVF router:

- Resolves `nlists = 0` to a deterministic small-table-friendly automatic
  list count.
- Selects deterministic sample indices with the existing seeded ChaCha8
  pattern used elsewhere in the repo.
- Normalizes finite, non-zero source vectors for spherical k-means.
- Trains bounded-iteration spherical k-means over normalized vectors.
- Handles `nlists > rows` by returning the requested centroid count with
  deterministic fallback seeding for empty clusters.
- Assigns query/source vectors to centroids by normalized inner product.
- Unit coverage checks auto-list resolution, stable sample selection, bad
  input rejection, unit normalization, deterministic training, more-lists-
  than-rows behavior, and centroid assignment separation.

## Review Focus

Please review for:

- Whether `ceil(sqrt(rows))` capped by row count and 4096 is the right first
  `nlists = 0` auto policy.
- Whether deterministic fallback seeding for empty clusters is acceptable
  before we have real-corpus recall gates.
- Whether the trainer should return exactly requested `nlists` even when
  `nlists > rows`, or clamp to row count and record fewer lists.
- Whether rejecting zero-norm rows is the right build behavior for the
  current inner-product surface.
- Whether normalized inner product tie-breaking by first centroid is good
  enough for deterministic bulk assignment.
- Whether this helper should move to a shared AM training module later, or
  stay IVF-local until another posting-list AM needs it.

## Non-Goals

This packet does not implement heap sample collection, PostgreSQL datum
decoding, populated `ambuild`, quantizer payload generation, posting-list
writes, WAL-safe list-directory updates, scan routing, recall measurement, or
planner costing.
