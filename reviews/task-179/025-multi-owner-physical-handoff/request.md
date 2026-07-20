---
agent: codex
role: coder
model: gpt-5
date: 2026-07-11
seq: 01
---

# Review request — Packet 025 multi-owner physical handoff

This checkpoint replaces the single-local build restriction with streamed
physical owner handoff over the immutable private participant bindings.

## Commit

- `c2929fcb5` — stream physical generations to remote owners.

## Transport and build

- Build requires exactly one local participant but accepts additional remote
  owners. Conninfo secrets resolve only after public descriptor/spec identities
  freeze and never enter canonical bytes or logs.
- The existing bounded owner router now dispatches each canonical batch by
  owner ordinal to local SPI or a pooled libpq participant connection.
- Remote begin/stage/seal use parameterized index locators, UUIDs, digests, and
  payloads. Stage responses independently decode accepted/cumulative counts and
  the 32-byte cumulative owner digest.
- The manifest collects one independently decoded Ready receipt per immutable
  participant in roster order.
- Coordinator abort now replays immutable local/remote bindings, aborts every
  unpublished participant generation, transitions the registration to
  `Aborted`, and releases retained build locks after commit.

## Live evidence

The PG18 transport fixture creates one coordinator and two remote participant
shells over distinct pooled loopback sessions. Thirty records reach three
nonempty Ready graph relations; pairwise vec-id intersections are empty and the
union is exactly the 30-row source. Coordinator abort then removes every
generation, clears the durable build gate, and permits DROP cleanup.

Validation and provenance are in `artifacts/manifest.md`.

## Explicit next work

- This is a three-owner transport integration fixture on one PG18 instance,
  not the required real three-instance fixture.
- Remote publication/T4b/retire dispatch and physical expansion/materialization
  remain open before end-to-end multi-instance reads.
- Persisted bounded head seeds and required 10k/50k/100k measurements remain
  open.

Leaving this request open for outside review.
