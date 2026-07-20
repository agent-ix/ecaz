---
agent: claude
role: coder
model: claude-opus-4-8
date: 2026-07-11
seq: 01
---

# Review request — Packet 011 build-to-Ready coordinator

Implements `ec_distann_build_epoch(index_regclass regclass, epoch bigint,
build_id uuid) RETURNS bytea` (FR-078:275-323), completing the coordinator trio
(`build_epoch` here, `abort_epoch_build` 009, `epoch_build_status` 010) for the
single-node lane.

## Commit

- `8b45d1cb` — `ec_distann_build_epoch` + `global_digests` + `capture_source_snapshot`.

Artifacts + provenance in `artifacts/manifest.md`.

## What it does (FR-078:275-323, 741-756)

In one coordinator transaction, after requiring the durable registration from
`begin_epoch_build` and reacquiring the frozen roster:

1. **One frozen source MVCC snapshot** via `capture_source_snapshot` — the active
   snapshot's 32-bit xids are widened to full wrap-safe ids against
   `ReadNextFullTransactionId` (matching PostgreSQL's
   `FullTransactionIdFromAllowableAt`), and the in-progress arrays are sorted
   into the canonical strictly-ascending order.
2. **Physical graph workspace** from `capture_physical_source_rows` +
   `build_physical_graph_workspace`.
3. **Counting/digest pass before the first begin**: `owner_expectations`
   (per-owner counts + stream digests) and `global_digests` (global record
   count + canonical global graph / row-tier digests, FR-078:741-747 exact byte
   layouts).
4. **Drives the local participant** begin → route/stage → seal, collecting the
   Ready receipt.
5. **Assembles and atomically persists** the immutable build candidate
   (`DistannBuildCandidateV1::from_components`, whose full cross-component
   consistency check over build-spec ↔ descriptor ↔ snapshot ↔ manifest ↔
   receipts passes), then transitions the registration to `Ready` and returns
   the 32-byte candidate digest.

The candidate is consumed by the later decision transaction rather than client
memory; the descriptor codec comes from the workspace, the roster from the
registration.

## Deliberately scoped / flagged for review

- **Single local participant only.** A multi-owner roster is rejected; multi-node
  remote begin/stage/seal transport is a later slice.
- **`head_sample_digest` is `[0u8; 32]`** in both the build spec and manifest.
  The publish-decision re-verification recomputes the candidate digest chain over
  the *stored* bytes (not re-derived from the graph), so this round-trips
  consistently; deriving the real coordinator head sample is a follow-up. Flagged
  for reviewer confirmation.
- **`DistannManifestCodecParameters`** is built for the RaBitQ artifact
  (GroupedPQ fields zero); `from_components` validates it against the descriptor
  artifact. Other codecs are exercised when their fixtures build to Ready.
- Snapshot/exact-replay idempotency (`EC_BUILD_ID_CONFLICT` on byte mismatch) is
  provided by `from_components` + the candidate PK/unique constraints; the
  dedicated replay test lands with the multi-owner slice.

## Validation

- `cargo check` + strict `cargo clippy` (`pg18 pg_test`, `-D warnings`) — pass at
  `8b45d1cb`.
- `cargo pgrx test pg18 test_distann_build_epoch_single_node` — 1/1 pass:
  end-to-end 3-row build to Ready, returned digest equals the persisted candidate,
  registration Ready, and `epoch_build_status` reports one Ready participant with
  3 records and a receipt.

Leaving the request open for outside review.
