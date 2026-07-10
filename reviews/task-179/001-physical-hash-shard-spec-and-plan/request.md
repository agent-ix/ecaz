# Task 179 packet 001 — physical hash-shard specification and implementation plan

**Status:** review requested. This packet specifies and plans the physical
hash-sharded generation work; it does not claim implementation or benchmark
completion.

**Branch:** `task-165-ec-distann-m3`
**Specification checkpoint:** `32b9b43fbdaa23cafb31ef25b759892c2c05028a`

## Outcome

The EC-DistANN distributed contract now requires one global graph whose graph
records and frozen row-tier tuples are physically disjoint by hash owner. A
logical coordinator root is metadata-only. Replicated complete graphs, replicated
row tiers, replica-count controls, and tombstone-pruned copies are invalid
substitutes for this topology.

Task 179 is the implementation task because Task 173 already reserves Tasks
173–178 for BatANN slices. Task 163 D8 remains a prerequisite, and the corrected
Task 172 benchmark gate is shelved until Task 179 provides the physical surface.

## Specification changes

- `FR-075` makes distributed-control mode explicit and fail-closed before a
  physical generation is published.
- `FR-076` and `FR-077` define canonical graph, handoff, source-row, and
  row-tier representations, including byte order, identities, and digests.
- `FR-078` defines the node registry, source identity and mutation gate,
  generation/build descriptors, trained codec artifacts, streamed participant
  staging, disjoint owner storage, and distinct pre-decision versus published
  topology selectors.
- `FR-079` fixes the query and row-materialization interfaces and bounds graph,
  exact-vector, and payload reads independently.
- `FR-082` defines the lifecycle state machine, 34-byte versioned epoch
  fingerprint, canonical manifest and Ready receipt, a durable coordinator
  decision boundary, recovery, generation pins, retirement, and force-retire
  audit behavior.
- `FR-083` binds tombstones, redirects, frozen row payloads, and reclamation to
  immutable generations.
- `NFR-014`, `NFR-016`–`NFR-020`, and `ADR-085` now carry the corresponding
  security, format, topology, storage, read-amplification, fault, and rationale
  requirements.

## Specification-review findings resolved

The base review caught and corrected several contract defects before code:

- the generation descriptor did not bind the complete trained codec artifact;
- scan lifetime was not protected by durable all-participant generation pins;
- Ready-generation topology could not be selected by an unpublished epoch
  fingerprint, so build-id and epoch-fingerprint selectors are now separate;
- the record-size acceptance formula counted a vector that is not in the
  canonical graph record;
- storage arithmetic treated landed RaBitQ as four-bit instead of one-bit;
- the read bound falsely equated graph expansions with live exact-vector reads;
- local heap TIDs were an unsafe fallback for global source identity.

`spec/reviews/base.md` records the full review and dispositions.

## Test-matrix changes

`spec/tests.md` now maps every amended acceptance criterion and constraint to
criterion-level verification. It adds:

- registry, descriptor, handoff, materialization, privilege, lifecycle,
  publication, pin, mutation-gate, retirement, format, and topology cases;
- explicit boundary, option-permutation, stable-error, state-transition, and
  edge-case coverage;
- a true three-PostgreSQL-instance integration fixture requirement;
- a physical-topology precondition for all Task 172 benchmark evidence;
- unique test identities: SPIRE retains `TC-020`, benchmark-suite coverage moves
  to `TC-049`, Task 173 retains `TC-045`–`TC-048`, and DistANN format discipline
  uses `TC-050`.

The matrix remains `PARTIAL` because no physical implementation or runtime
evidence exists yet.

## Task and status reconciliation

- Task 163 remains partial/open for D8 streaming.
- Tasks 164 and 165 are useful distributed-control/read-path work, but do not
  constitute physical sharding.
- Task 166 evidence is explicitly a single-instance control.
- Task 167 must adapt DML ownership and redirects to physical generations.
- Task 172 is shelved until the Task 179 implementation checkpoint, then must
  produce suite-driven 10k/50k/100k A/B recall, latency, and storage evidence.
- BatANN Task 173 may proceed with B0, but B1/B3/B4 must consume the Task 179
  generation contract and replace its draft 16-byte fingerprint assumption with
  the canonical 34-byte versioned fingerprint.

## Implementation plan

`plan/tasks/179-ec-distann-physical-hash-shard-generations.md` is deliberately an
implementation plan, not a second specification. Its first implementation uses
ordinary AM-owned PostgreSQL heap/B-tree relations per generation so staging can
commit, abort, recover, and reclaim transactionally:

- frozen row-tier heap with the source tuple descriptor;
- graph heap keyed by vector identity and pointing to the row-tier TID;
- unique B-tree directory for vector identity lookup;
- cataloged generation, participant, receipt, decision, pin, and audit state.

It divides delivery into narrow checkpoints for D8 input, local generation
storage, canonical row capture and streamed handoff, participant endpoints,
coordinator publication/recovery, pins/retirement, read integration, and the
true multi-instance fixture.

## Reviewer focus

Please review these high-risk seams:

1. The three committed publication phases: build-to-Ready, durable decision,
   then recoverable participant publish and coordinator pointer swap.
2. The session source lock plus durable mutation gate and its fail-closed DDL/DML
   coverage.
3. Descriptor/receipt/manifest digest binding, especially trained codec and row
   schema identity.
4. Durable scan pins across every participant and partial cleanup after failure,
   cancellation, or restart.
5. The distinct build-id and epoch-fingerprint topology selectors.
6. The ordinary heap/B-tree first implementation and whether it preserves the
   specified ownership, atomicity, WAL, and reclamation semantics.

## Validation

- `quire validate --scope /home/peter/dev/ecaz-task165 "spec/**/*.md" --summary`
  — `244/244 docs grammar-clean (100%); 0 EARS finding(s): none`.
- Stable EC error categories referenced by the amended specs but absent from the
  matrix: `0`.
- Duplicate first-column test-summary identifiers: `0`.
- `git diff --check -- spec plan`: pass before the specification checkpoint.
- No build, PostgreSQL test, or benchmark was run because this checkpoint changes
  specification, traceability, and planning only.

## Known open work

- Task 179 has no implementation yet.
- Task 163 D8 streaming remains open.
- BatANN Task 173 needs the noted fingerprint/dependency reconciliation on its
  branch.
- Task 172 remains non-promotable until the physical implementation and complete
  10k/50k/100k suite evidence exist.
