# Artifact manifest

- Task bucket / packet: `reviews/task-172/004-unshelve-readiness`
- Evidence type: immutable cross-packet readiness audit; no new measurement
- Review base SHA: `2963756ab8701adc5c2afb200def2d7e3678efb0`
- Branch: `task-203-ec-distann-conformance`
- Created: `2026-07-29` (America/Los_Angeles)
- Lane / fixture / storage format / rerank mode: not applicable; this packet
  cites accepted evidence and does not run a lane
- Isolation surface: not applicable

## Source condition

`plan/tasks/172-ec-distann-real-multinode-benchmark-gate.md` requires Task 179's
physically sharded lane to exist and pass TC-040, TC-042, TC-050, and the
fail-closed topology audit before Task 172 may be unshelved.

## Immutable evidence citations

### Task 179 status and aggregate acceptance

- `plan/tasks/179-ec-distann-physical-hash-shard-generations.md`
  - status `done`;
  - physical fixture and accepted matrix no longer block Task 172;
  - acceptance criteria map TC-050, physical topology, lifecycle/fault, and
    physical read behavior to the delivered packet groups.
- `reviews/task-179/059-closeout/feedback/2026-07-13-01-reviewer.md`
  - all 13 acceptance criteria substantively met;
  - AC-2 / TC-050 done;
  - exact/disjoint physical topology done;
  - Task 179 aggregate accepted with three mechanical conditions.
- `reviews/task-179/060-recovery-state-closeout/feedback/2026-07-13-01-reviewer.md`
  - all three packet-059 conditions closed;
  - 238 passing DistANN PG18 tests, zero failures, and 21/21 on-disk fixtures;
  - Task 179 done status stands.
- `reviews/task-179/060-recovery-state-closeout/artifacts/manifest.md`
  - exact commands, source SHAs, and checksums for the aggregate PG18 and
    on-disk-format evidence.

### Physical fixture and topology gate

- `reviews/task-179/031-real-physical-multicluster/request.md`
  - physical source exists only on the coordinator;
  - owner generations are disjoint;
  - old replicated fixture is retained only as an explicitly non-gate control;
  - absent or failed topology prevents accepted downstream evidence.
- `reviews/task-179/031-real-physical-multicluster/feedback/2026-07-12-01-reviewer.md`
  - outside approval of the genuine multi-process physical fixture;
  - zero source rows on participants, no pruning, exact count coverage, zero
    non-owner/orphan residue, and in/out-roster shapes.
- `reviews/task-179/031-real-physical-multicluster/artifacts/manifest.md`
  - suite config and 3/3 passing topology thresholds.
- `reviews/task-172/002-physical-multinode-benchmark/feedback/2026-07-12-01-reviewer.md`
  - topology accepted as decision-grade at 10k/50k/100k:
    exact global coverage, zero non-owned rows, zero orphans, disjoint shards,
    and remote owner materialization.
- `reviews/task-172/003-postfix-physical-matrix-acceptance/artifacts/manifest.md`
  - final physical matrix provenance and post-fix evidence chain.

### TC-040 / TC-042 / TC-050

- `reviews/task-179/005-streamed-handoff/`
  - transactional streamed handoff, physical rescan/seal, replay, independent
    owner-stream state, and format fixtures.
- `reviews/task-179/053-physical-publish-fault-windows/`
  - suite-driven real three-process lifecycle faults; accepted in
    `feedback/2026-07-14-01-reviewer.md`.
- `reviews/task-179/002-format-and-control/`
  - fourteen golden fixtures, independent decoders, byte-swap/unknown-version
    rejection, layout assertions, and upgrade-matrix coverage.
- `reviews/task-179/006-publication-and-retention/`
  - descriptor-v2 and lifecycle format fixtures; 83 fixture/layout/upgrade
    tests passing.
- `reviews/task-179/034-cancelled-generation-recovery/`
  - cancellation-audit production round trip, independent decoder,
    endian/version rejection, golden fixture, offsets, and upgrade row.

## Commands

Repository refresh:

```text
git pull --ff-only origin main
```

Result: `Already up to date.`

Evidence inspection used read-only `rg`, `sed`, `find`, `git log`, and
`git diff` commands against the cited files. No test, benchmark, corpus, or
cluster command was run, and no new raw artifact was generated.

## Caveat

`spec/tests.md` retains stale `Planned` labels for TC-040, TC-042, and TC-050.
The packet treats that as a traceability reconciliation item for Task 203/208,
not as contrary runtime evidence.
