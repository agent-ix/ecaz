# Task 203: ec_distann Decision Re-Audit and Paper Conformance

Status: **in progress** (2026-07-29). Priority: P0 program-integrity audit.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`.
Governing requirements: `NFR-021`, `NFR-022` (landed by this task's spec slice),
`NFR-017`, `NFR-018`, `FR-078`, `ADR-067`.

## Why

ec_distann implements `DISTRIBUTEDANN` (arXiv:2509.06046). An audit on
2026-07-29 found the program had drifted from that design on four independent
axes, and that the drift was self-reinforcing: each step was locally reasonable
and no single task's gate could see it.

1. **The traversal regime was never applied.** Task 162's G0 kill-check — the
   measurement that unblocked the program — concluded "wide beam, few rounds is
   the only viable multinode shape ... multinode wants >=32", measuring BW=32/H=8
   at 0.9940 recall / 20.3--28.3 ms projected against BW=4/H=64 at 77.6--141.6 ms
   ("far over"). The default was never changed. Every suite from Task 179 onward
   is pinned at BW=4/H=100. BW=4 was inherited from `ec_diskann`, tuned in Task
   168 to fill 32-wide local SIMD kernels with no network in the loop.
   `ECDISTANN_MAX_BEAM_WIDTH = 64` makes the paper's grid unreachable even as a
   session GUC, and BW >= 32 has never been run on the distributed path.
2. **The pushdown that makes wide beams affordable is absent.** Paper Algorithm 1
   pushes threshold `t` and candidate limit `l` to each storage host, which
   prunes before returning. In ecaz `code_threshold` is hardcoded `None` at the
   only call site, discarded entirely by the production expander, `l` does not
   exist, and owners return every neighbor unsorted and untruncated. Tasks 188
   and 194 therefore tested beam widening without its enabling mechanism.
3. **The head diverges from §2.2/§3 on every axis.** Tasks 181 and 185
   established that head *membership* bounds recall (0.9625 vs the 0.9970
   same-graph oracle) and that three different 4,096-row objectives produced
   identical top-32 seeds. §3 names the cause — build the head "from the union of
   the top layers of each partition's graph, rather than the top layers of the
   stitched-together graph" — and ecaz builds from the stitched global graph. The
   head is also not sharded (§2.2) and not replicated (§4.1).
4. **The replica abandoned the distributed premise.** Tasks 198/199 promoted a
   coordinator-resident full-graph replica and it became the program's recorded
   latency control, so forward work inherited a control in which search is not
   distributed.

These interlock. BW=4 forces ~10 sequential rounds; ten rounds produce the
transport wait that motivated Task 190's architecture escalation; the escalation
produced the replica, which removed transport wait by removing distribution.

## Goal

Re-audit every ec_distann decision (**Tasks 161--167, 172, 179--202**) against
the restored requirements, and record for each whether its disposition survives.
Produce verdicts and recommendations only. This task changes no production
behavior, no task status line, and no `src/**`.

## Method

Every task is classified on four tests.

| Test | Question |
| --- | --- |
| T1 Control validity | Was the decision's A/B control itself DistANN (`NFR-022`)? |
| T2 Distribution invariant | Does the change, or the evidence supporting it, require O(N) single-node state (`NFR-021`)? |
| T3 Regime validity | Was it measured at BW=4/H=100, and/or without the Algorithm 1 pushdown — a parameter point the G0 kill-check called non-viable, missing its enabling mechanism? |
| T4 Evidence integrity | Does the cited evidence exist, at the claimed scales, on a conforming lane, with the metric actually able to express the claim (`NFR-007`, `NFR-018`)? |

Verdicts: **SOUND** (disposition stands), **SUSPECT-REMEASURE** (finding may
stand but the evidence cannot support the disposition as written), **INVALID**
(disposition rests on an inadmissible control or unmeasurable evidence).

A verdict is recorded only with a citation to a real packet path and artifact.
No verdict rests on prose alone.

## Cross-cutting checks

- **Arm-blind storage sweep.** The suite's storage step computes its scalars
  before the arm loop (`crates/ecaz-cli/src/commands/dev/distann_multicluster.rs:5153-5160`)
  and reprints them inside it (`:5209-5212`), so arms are byte-identical by
  construction. Every packet claiming "storage identical/unchanged between arms"
  asserted something the metric cannot express. Re-classify those rows across all
  audited tasks, not only 198/199.
- **NFR-018 ratio row.** `NFR-018` requires the ratio row in the packet manifest
  per scale. `cluster_index_space_amplification` exists at
  `distann_multicluster.rs:7419-7482` and ran for Tasks 172 and 197. Record every
  gate packet that omitted it.

## Deliverables

1. `reviews/task-203/001-decision-reaudit/` — the full 161--202 decision matrix,
   the four defects with citations, and per-task recommendations.
2. Cross-link feedback files in the buckets whose dispositions are challenged.
3. The `NFR-021`/`NFR-022`/`NFR-018`/`StR-008`/ledger spec slice (landed).

## Non-goals

- Changing any task's `Status:` line. The audit records verdicts; the operator
  decides what moves.
- Deleting or disabling FR-084, the traversal replica, or any code. Disposition
  of the replica is a separate operator decision.
- Implementing the conformance program. Each gap becomes its own numbered task
  with its own A/B under `NFR-021`/`NFR-022`.
- Re-running benchmarks. The audit is a re-read of committed evidence;
  re-measurement is triggered only for tasks it marks SUSPECT-REMEASURE.

## Follow-on program (proposed; not created)

Sequencing is **pushdown -> regime -> head**: neither the wide-beam question nor
the seed-width question can be answered without the pushdown in place.

- Algorithm 1 threshold/limit pushdown and owner-side prune/sort/truncate.
- Traversal regime: raise the beam ceiling and sweep the paper's shape on the
  distributed path.
- Head reconstruction per §2.2/§3: per-partition union, sharded, ANN-searched.
- Bounded degraded completion per §4.2, under `NFR-020-AC-6`'s opt-in clause.
- `NFR-021`/`NFR-022` conformance gates in `ecaz bench suite`.
- Storage-step arm fidelity in the multicluster runner.

## Numbering note

**Task 201 is double-allocated.** `origin/main` carries
`201-task38-interrupt-poll-followups.md`; the ec_distann lane carries
`201-ec-distann-post-replica-latency-residual.md` (commit `c830b184f`, not yet
merged to main). Cite the ec_distann 201 by explicit branch and path, per the
convention StR-008 already uses for the 141--146 collisions. Task 203 and above
are free on both lanes as of 2026-07-29.

## References

- `DISTRIBUTEDANN`, arXiv:2509.06046 §2.2, §2.3, §2.4, §3, §4, §4.1, §4.2.
- `ADR-067:47-51` (storage scale-out as the defining property), `ADR-085`,
  `ADR-086`.
- `NFR-017`, `NFR-018`, `NFR-019`, `NFR-020`, `NFR-021`, `NFR-022`, `StR-008`.
- `reviews/task-162/003-g0-killcheck/` (the unapplied regime finding).
