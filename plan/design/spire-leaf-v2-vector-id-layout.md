# SPIRE Leaf V2 Vector ID Layout

Status: active Phase 11.2 storage decision
Task: Task 30 Phase 11

## Decision

Leaf V2 keeps one fixed-width vector-ID column per leaf object, but the meta
row now controls the column namespace:

- `LocalU64` rows keep the existing 16-byte padded local layout:
  discriminator `0x01`, little-endian `u64`, zero padding. Existing local-only
  objects remain byte-compatible.
- `GlobalBytes` rows store raw `SpireVecId` bytes with discriminator `0x02`.
  Every row in one Leaf V2 object must have the same byte length, and that
  length is recorded as `vec_id_stride`.
- Mixed local/global rows and variable-width global rows inside one Leaf V2
  object are rejected.

The V2 format already had `vec_id_kind` and `vec_id_stride` fields. This slice
activates them instead of adding a new partition-object version.

## Rationale

Leaf V2 has no per-row vector-ID length prefix. Padding global IDs to the
largest row would make trailing zero bytes ambiguous because those bytes could
also be part of the real global payload. A fixed-width global column avoids
that ambiguity, keeps segment row sizing predictable, and still supports the
Phase 11 writer contract once source identities are encoded as fixed-width
payloads.

Keeping local and global namespaces separate per object also prevents a class
of merge bugs where a local sequence and a global payload accidentally share
storage and dedupe semantics. Cross-node merge can still support local-only
indexes by scoping local IDs with `node_id`; global-ID objects dedupe by the
raw `0x02` bytes.

## Consequences

- Writer/build paths that want global base objects must supply fixed-width
  stable source payloads.
- Delta assignment rows keep their existing variable-length row encoding and
  can continue to carry any valid `SpireVecId`.
- A future Leaf V3 or length-prefixed Leaf V2 extension is required if we need
  mixed namespaces or variable-length global IDs within one base object.
- Candidate scanning and assignment-row reconstruction must decode vector IDs
  through the column meta instead of reconstructing local IDs from the padded
  sequence column.
