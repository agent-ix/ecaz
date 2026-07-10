# Task 179 packet 002 — physical generation formats and control metadata

**Status:** review requested. This packet implements the format/control
checkpoint and incorporates every finding from the outside review of packet
001. It does not claim physical generation storage, multi-node handoff, or
benchmark completion.

**Branch:** `task-179-ec-distann-physical-shards`

**Code checkpoint:** `04c1ce70b7483c6b5828cbfc7b388c3ac668d680`

**Review-finding correction:** `57a45cca1afba081563b33f253dc9d89a4826f08`

## Outcome

The opt-in distributed-control index now has a persisted metadata-only identity
and fail-closed behavior, while the physical generation protocol has one
canonical implementation for every format that later storage/handoff work must
persist or exchange. Existing `distributed_control=false` indexes retain their
byte-identical writable v4 metadata and legacy behavior.

## Implemented scope

- Added the `distributed_control` reloption. A true value creates permanent,
  WAL-logged v5 metadata containing a never-zero RFC 4122 version-4 logical
  index UUID and no graph state. Temporary/unlogged control indexes, missing
  global source identity, direct scans before publication, and DML pending Task
  167 fail closed.
- Added bounded little-endian canonical codecs for row-schema descriptors,
  trained codec artifacts, ordered generation descriptors, build
  specifications, handoff entries/batches, wrap-aware source snapshots, Ready
  receipts, manifest-v2 subrecords/manifests, and 34-byte epoch fingerprints.
- Added the persisted physical graph-record v1 encoding with the exact 20-byte
  header and six-byte `ItemPointer` representation required by FR-076.
- Added generation-side quantizer restoration so seeded and trained codec
  artifacts reconstruct the same scoring implementation without owner-side
  retraining.
- Pinned placement-hash v1 as wrapping MurmurHash3 `fmix64` with its published
  domain constant, five hash vectors, and three-owner results.
- Added fourteen golden fixtures, independent decoders, byte-swap/unknown-version
  rejection, size/offset assertions, and upgrade-matrix rows. The follow-up
  changed the snapshot fields to FullTransactionId-width and regenerated every
  transitively affected fixture.
- Added an executable traceability audit and amended the specification/task plan
  for the outside review's build-registration, retention, recovery, privilege,
  storage-accounting, identity, and terminology findings. The item-by-item
  disposition is packet 001 feedback sequence 02.

## Outside-review disposition

All three P1s, all nine P2s, and every P3 item in Claude's packet 001 feedback
have an explicit code/spec/task or artifact correction. In particular, the
original durable participant pin protocol was not retained: scans now use a
coordinator-local registry, while retirement performs the durable fenced
cluster operation. Outside re-review is requested; the coder has not closed the
request.

## Validation

- `cargo test --lib --no-default-features --features pg18 distann`:
  **139 passed, 0 failed, 1 ignored**. The ignored test is the intentional
  golden-fixture emitter.
- On-disk/upgrade integration: **65 fixture tests + 13 layout tests + 2 upgrade
  matrix tests passed**, with zero failures.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D
  warnings`: pass.
- `cargo pgrx test pg18 test_distann_control`: **3 passed, 0 failed**, covering
  v5 metadata/fail-closed behavior, required include identity, and permanent
  WAL-logged persistence.
- Packet 001's refreshed Quire log is clean at 244/244, and its reproducible
  trace audit reports zero missing error categories and zero duplicate test IDs.

All command output and exit statuses are packet-local under `artifacts/`.

## Reviewer focus

1. Canonical byte layouts, length/count bounds, digest domain separation, and
   decode-before-allocation behavior.
2. The v4/v5 compatibility boundary and whether control metadata can ever be
   mistaken for a local graph.
3. Complete trained-codec binding/restoration and manifest/build-spec field
   agreement.
4. FullTransactionId snapshot identity and the regenerated transitive fixtures.
5. Placement-hash implementation/spec parity and golden owner results.
6. Whether every outside-review disposition is actually reflected in the
   normative spec/test matrix, not just this response.

## Explicitly out of scope / still open

- Transactional hidden generation relations and catalogs.
- Remote begin/stage/seal/abort handoff and source-row capture.
- Publication, coordinator scan retention, retire recovery, and active pointer.
- Generation-aware expansion/materialization and Task 167 physical DML.
- The true three-PostgreSQL-node disjoint-owner fixture.
- Required suite-driven 10k/50k/100k A/B recall, latency, and cluster-storage
  evidence. No benchmark result is claimed by this packet.
