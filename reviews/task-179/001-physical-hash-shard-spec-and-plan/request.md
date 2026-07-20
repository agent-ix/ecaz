# Task 179 packet 001 — physical hash-shard specification and implementation plan

**Status:** outside re-review requested after findings were addressed. This
packet specifies and plans the physical hash-sharded generation work; it does
not claim physical-topology or benchmark completion. Format/control code is
reviewed separately in packet 002.

**Branch:** `task-179-ec-distann-physical-shards`

**Original specification checkpoint:** `32b9b43fbdaa23cafb31ef25b759892c2c05028a`

**Outside-review correction:** `57a45cca1afba081563b33f253dc9d89a4826f08`

## Outcome

The EC-DistANN distributed contract now requires one global graph whose graph
records and frozen row-tier tuples are physically disjoint by hash owner. A
logical coordinator root is metadata-only. Replicated complete graphs, replicated
row tiers, replica-count controls, and tombstone-pruned copies are invalid
substitutes for this topology.

Task 179 is the implementation task because Task 173 already reserves Tasks
173–178 for BatANN slices. The Task 163 D8 prerequisite has landed and received
outside acceptance; the corrected Task 172 benchmark gate remains shelved until
Task 179 provides the physical surface.

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
  fingerprint, canonical manifest and Ready receipt, durable publish/retire
  decisions, recovery, coordinator-local scan registration, retirement
  fencing, and force-retire audit behavior.
- `FR-083` binds tombstones, redirects, frozen row payloads, and reclamation to
  immutable generations.
- `NFR-014`, `NFR-016`–`NFR-020`, and `ADR-085` now carry the corresponding
  security, format, topology, storage, read-amplification, fault, and rationale
  requirements.

## Specification-review findings resolved

The base review caught and corrected several contract defects before code:

- the generation descriptor did not bind the complete trained codec artifact;
- the initial all-participant durable-pin correction imposed an unacceptable
  per-query RPC/commit tax and was replaced after outside review by local scan
  registration plus a durable retirement fence;
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
  publication, scan-retention, mutation-gate, retirement, format, and topology
  cases;
- explicit boundary, option-permutation, stable-error, state-transition, and
  edge-case coverage;
- a true three-PostgreSQL-instance integration fixture requirement;
- a physical-topology precondition for all Task 172 benchmark evidence;
- unique test identities: SPIRE retains `TC-020`, benchmark-suite coverage moves
  to `TC-049`, Task 173 retains `TC-045`–`TC-048`, and DistANN format discipline
  uses `TC-050`.

The matrix remains `PARTIAL` because physical generation storage/topology and
its runtime evidence do not exist yet.

## Task and status reconciliation

- Task 163 D8 streamed stitching is landed and outside-review accepted.
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
- cataloged generation, batch, build-registration, receipt, publish/retire
  decision, active-pointer, and audit state.

It divides delivery into narrow checkpoints for D8 input, local generation
storage, canonical row capture and streamed handoff, participant endpoints,
coordinator publication/recovery, scan retention/retirement, read integration,
and the true multi-instance fixture.

## Reviewer focus

Please review these high-risk seams:

1. The four committed phases: durable build registration before remote begin,
   build-to-Ready, durable publish decision, then recoverable participant
   publish and coordinator pointer swap.
2. The session source lock plus durable mutation gate and its fail-closed DDL/DML
   coverage.
3. Descriptor/receipt/manifest digest binding, especially trained codec and row
   schema identity.
4. Coordinator-local scan registration with zero participant query-path writes,
   plus the zero-in-flight durable retire decision and partial-apply recovery.
5. The distinct build-id and epoch-fingerprint topology selectors.
6. The ordinary heap/B-tree first implementation and whether it preserves the
   specified ownership, atomicity, WAL, and reclamation semantics.

## Validation

- `quire validate --scope /home/peter/dev/ecaz/.claude/worktrees/task-179-physical-shards "spec/**/*.md" --summary`
  — `244/244 docs grammar-clean (100%); 0 EARS finding(s): none`.
- Stable EC error categories referenced by the amended specs but absent from the
  matrix: `0`.
- Duplicate first-column test-summary identifiers: `0`.
- `git diff --check -- spec plan`: pass at the outside-review correction head.
- Packet 001 remains a specification/traceability checkpoint. Packet 002 owns
  the format/control build, clippy, fixture, and live PG18 evidence; neither
  packet claims a benchmark result.

## Known open work

- Task 179 now has its format/control checkpoint, but generation storage,
  handoff, publication/retention, read integration, and the physical fixture
  remain open.
- Task 163 D8 is complete and supplies Task 179's bounded stitched input.
- BatANN Task 173 needs the noted fingerprint/dependency reconciliation on its
  branch.
- Task 172 remains non-promotable until the physical implementation and complete
  10k/50k/100k suite evidence exist.
