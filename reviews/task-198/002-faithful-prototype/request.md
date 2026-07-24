---
task: 198
packet: 002-faithful-prototype
head_sha: b9398a1ba
role: coder
date: 2026-07-23
status: open
---

# Review request: faithful coordinator traversal-replica prototype

Please review commit `b9398a1ba`.

## Scope

- Streams every owner graph record and exact source vector into one
  coordinator-local, WAL-logged derived relation. Payload columns remain
  owner-side.
- Revalidates candidate, descriptor, fingerprint, owner placement, record
  shape, dimensions, codec, cardinality, owner coverage, and canonical global
  and per-owner digests before `Ready`.
- Opens only an exact identity-matching `Ready` image after the existing
  physical-generation pin and runs the unchanged orchestration core, RaBitQ
  neighbor scoring, and exact final score locally.
- Preserves owner ordinals and local heap TIDs; remote results retain an
  invalid TID and therefore use the unchanged owner payload path.
- Restarts from the beginning on the owner traversal path after any replica
  open or mid-traversal failure. No partial replica frontier or hit is reused.
- Adds one `ecaz bench suite` variant axis. It builds the image once and emits
  content digest, copy bytes, relation bytes, WAL, build time, replica-local
  graph/vector and score stages, and fallback counts.
- Adds the lifecycle primitives required for the next packet: autonomous
  Ready-to-Stale mutation guard and separately fenced retire/reclaim.

## Validation

See [`artifacts/manifest.md`](artifacts/manifest.md).

- PG18 feature build/check passed.
- Three replica identity/state unit tests passed.
- Thirty-one DistANN CLI/suite tests passed.
- `git diff --check` passed.

Runtime lifecycle/fault drills and the isolated 100k A/B intentionally land in
packets 003 and 004 before any decision.
