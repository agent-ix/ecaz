# Review request — Task 212 P0: crown cache spec

- Task: `plan/tasks/212-ec-distann-crown-cache.md`, phase P0 (spec-first)
- Packet: `reviews/task-212/001-crown-cache-spec/`
- Spec artifact: `spec/functional/distann/read/FR-089-distann-crown-cache.md`
  (commit `9c85d3fed`, branch `task-203-ec-distann-conformance`)
- Date: 2026-08-01. Coder: fable (Claude Fable 5)

## What to review

FR-089 specs the crown: a fixed-capacity coordinator navigation cache over a
subset of head landmarks, entries `(vec_id, quantized code)` only, capacity
independent of N **and** C, refusal-not-eviction, epoch-fingerprint-keyed,
per-backend, lazily populated by bounded owner RPCs, rebuild-only (D10 —
head membership frozen within an epoch), discarded on capacity change,
static deterministic attested selection (frequency-aware admission
explicitly out of scope), miss ⇒ full sharded fan-out with identical
results. FR-084 bright line stated: the crown narrows the distributed
protocol, never substitutes for it.

Landed against the elevated structure, downstream of FR-086 (gateway
copies), whose bounded codes-only class and staleness rule the crown
inherits — the lineage the Task 210 round-2 review asked for (003a
question 2's "bounded coordinator-side cache" is exactly this artifact).

Design points worth challenge:

1. **Width pruning without fusion** — FR-089 allows crown-scored fan-out
   narrowing but binds A/B honesty: without FR-090 the win is owner CPU and
   tail width, not the RTT. The task file's P2 phase matches.
2. **Per-backend, no shared memory** — matches the FR-080 cache posture;
   a shared-memory crown would need a different conformance argument.
3. **Counter names** (`crown_seeds_served`, `crown_fallbacks`) asserted
   non-zero in candidate arms from day one (the Task 210 inert-mechanism
   lesson, specced as a requirement per user ruling).

## Validation

`quire validate` clean (advisory EARS warnings only). Implementation
(P1 structure/population/counters) NOT started.

## Status

Open — awaiting reviewer feedback.

## Update (2026-08-01, same session)

The spec was subsequently hardened by the failure-domain, integrity, and
scope-boundary analyses (`spec/reviews/{failure-domain,integrity,scope-boundary}.md`,
findings + resolutions recorded there). Material additions: see the
resolution lists for the FR this packet reviews.
