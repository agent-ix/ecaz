# Task 203 / 001 — ec_distann Decision Re-Audit

- Task: 203 (ec_distann decision re-audit and paper conformance)
- Branch: `task-203-ec-distann-conformance`
- Date: 2026-07-29
- Scope: Tasks 161--167, 172, 179--202
- Spec slice landed with this packet: `NFR-021`, `NFR-022`, `NFR-018` per-node
  term, `StR-008` conformance precondition, roadmap ledger corrections

## Summary

ec_distann has drifted from `DISTRIBUTEDANN` (arXiv:2509.06046) on four
independent, mutually reinforcing axes. The drift is not attributable to any
single bad decision: each disposition cleared the gates in force at the time.
What failed is that the gates could not express the properties being traded away.

The chain is causal, not coincidental:

> BW=4 forces ~10 sequential hop rounds -> ten rounds produce 5.013 ms transport
> wait -> transport wait is the measured residual that activated Task 190's
> architecture escalation -> the escalation selected a coordinator traversal
> replica -> the replica removed transport wait by removing distribution.

It begins at a parameter that the program's own entry gate measured as
non-viable, and it was never corrected because the parameter was never re-tested
on the distributed path.

**Recommendation:** re-open the dispositions marked INVALID below; treat the
SUSPECT-REMEASURE findings as open rather than settled; retain the SOUND
dispositions. Sequence the corrective program **pushdown -> regime -> head**.

---

## Defect 1 — The traversal regime was never applied

Task 162's G0 kill-check is the measurement that unblocked the entire program.
`reviews/task-162/003-g0-killcheck/request.md:10-26`:

> "**Wide beam, few rounds is the only viable multinode shape.** BW=4 needs H=64
> for 0.995 -> projected 78-142 ms (dead). BW=32 reaches 0.994 at H=8. **This
> matches the DistributedANN paper's regime** and should inform the M2 defaults
> (beam_width default 4 is a single-node default; multinode wants >=32)."

`reviews/task-162/003-g0-killcheck/artifacts/manifest.md:63-68`:

| Config | recall@10 | compute p50 | projected multinode | verdict |
| --- | ---: | ---: | ---: | --- |
| BW=32, H=8 | 0.9940 | 12.3 ms | 20.3--28.3 ms | under, 1.3--1.9x headroom |
| BW=32, H=16 | 0.9965 | 17.4 ms | 33.4--49.4 ms | borderline |
| BW=4, H=64 | 0.9950 | 13.6 ms | 77.6--141.6 ms | far over — not viable |

**The default was never changed.** `src/am/ec_distann/mod.rs:253` is still
`ECDISTANN_DEFAULT_BEAM_WIDTH = 4`; `:260` is `HOP_ROUNDS = 100`. Every distann
suite JSON from Task 179 onward — including the Task 198/199 replica A/B — is
pinned at BW=4/H=100. The GUC description at
`src/am/ec_distann/options.rs:331` still reads "The default is provisional until
the M0 recall-vs-H kill-check measurement pins it"; the kill-check ran
2026-07-07 and nothing was pinned.

Provenance of BW=4 — `src/am/ec_diskann/options.rs:29-32`:

> "Default from the Task 168 packet 002 A/B: width 4 wins latency at every
> 50k/100k sweep point (up to 18%)..."

That A/B (`reviews/task-168/002-batched-beam-ab/request.md:20`) swept beam width
to fill 32-wide **local SIMD scoring kernels, with no network in the loop**.
`ec_distann` copied the constant. A cache line and a network round trip were
tuned with the same number.

Structural limits, none of which appear in any recorded rationale:

- `ECDISTANN_MAX_BEAM_WIDTH = 64` (`mod.rs:254`) makes the paper's grid
  (BW 96--192; production BW=128) unreachable **even as a session GUC**.
- `top_k` default 10 vs the paper's k=L=200. `crates/ecaz-cli/src/profiles.rs:218-235`
  treats `top_k`, not BW/H, as the quality axis, so BW/H are not sweep axes at all.
- Seed count is `max(BW*2, 32)` (`generation_read.rs:2650`) vs k_head=200.
- **BW >= 32 has never been run on the distributed path.** The only BW=32 rows in
  the repo are single-node 50k from the 2026-07-07 kill-check, before remote code
  existed.
- The one wide-beam distributed test (Task 179 packet 066, BW16/H25) held
  `BW x H = 400` **fixed** — it traded rounds for width at constant work. No
  experiment has ever raised the budget the way the paper does (BW=128 x H=5 =
  640 expansions, k=L=200, R=72).

A reviewer raised this independently a week after the kill-check —
`reviews/task-179/060-recovery-state-closeout/feedback/2026-07-13-01-reviewer.md:112-117`:

> "default `BW=4` (`mod.rs:245`) makes per-owner RPCs carry 1-2 vec_ids, so
> overhead ~ rounds x per-RPC fixed cost. ... plausibly the largest single win."

It produced the fixed-product BW16/H25 run and nothing further.

## Defect 2 — The pushdown that makes wide beams affordable is absent

Paper §2.3 Algorithm 1 runs on each storage host and receives a threshold score
`t` and candidate limit `l`; §2.4 supplies `t = peek_worst(H_C)` each round; the
host prunes and truncates before returning. §2.3 equation (2) quantifies the
resulting ~6x bandwidth saving, and the paper adds: "We increase the savings
further by pruning any neighbors that are worse than the current worst member of
the candidate heap before returning to the orchestration service."

In ecaz:

| Mechanism | Status |
| --- | --- |
| threshold `t` on the wire | Parameter exists; **hardcoded `None` at the only call site** — `src/am/ec_distann/scan.rs:215`, `expander.expand_nodes(&batch, None)`. No production or test call site passes `Some(...)`. |
| threshold honored owner-side | **Discarded by the production expander** — `generation_read.rs:3146-3149`, `_code_threshold`. Also discarded by the replica (`traversal_replica.rs:2455`). Honored only by the legacy `LocalNodeExpander` (`expand.rs:127-137`), reachable only with `None`. |
| candidate limit `l` | **Does not exist** in FR-079, FR-081, any Rust struct, or any SQL overload. |
| `peek_worst(H_C)` feedback | **No counterpart.** `scan.rs` never reads its beam's worst member; only its best unvisited member, for the convergence early-exit. |
| owner-side prune / sort / truncate | **Not implemented.** Owners return every neighbor of every requested node, unsorted and untruncated; the coordinator does 100% of heap management. |

`FR-079:115-123` disarmed it deliberately, resolving failure-domain finding
FND-006:

> "`code_threshold` SHALL default to NULL (no pruning). ... a documented
> recall-risk optimization **outside the scan's correctness guarantees** ...
> never used where correctness or the gate is asserted."

That decision is defensible on its own terms. What was not recorded is the
coupling: it removes the mechanism that makes the paper's beam width affordable.
Note also that the paper's threshold is derived from the coordinator's *own*
candidate heap, so it prunes only what the beam would have discarded — a
materially different claim from FR-079's "may prune true results". The
recall-equivalence argument is available and was never made.

**Consequence for Tasks 188 and 194:** both tested BW=8 without pushdown. At
BW=128 with degree 32, ecaz would ship ~4,096 untruncated `(id, score)` pairs per
owner per round where the paper ships only those passing `t`, truncated to `l`.
Task 194 packet 007's recorded signature — hops 10.0->5.88, transport wait
-0.744 ms, but expanded nodes 40.0->47.04 and straggler spread 0.411->0.736 ms —
is the predicted result of widening without pushdown, not evidence against wide
beams.

**What does conform:** owners genuinely perform near-data scoring and return
scores rather than nodes (`generation_read.rs:3182-3190`); exact distance is
computed owner-side against the co-placed full-precision vector
(`FR-079:97-106`); batch node reads and per-owner concurrent fan-out are present;
FR-076's embedded neighbor codes implement §2.2's duplication trade correctly.

## Defect 3 — The head index diverges from §2.2 and §3 on every axis

Tasks 181 and 185 independently established the controlling fact: **head
membership, not head search, bounds recall.** `reviews/task-181` shows exact
scoring of the cap-4096 sample returning identical 0.9275 recall to graph search,
while the same-graph owner oracle reaches 0.9970 at 2445 ms.
`plan/tasks/181-...md:27-30`:

> "**Exact scoring cannot select useful entry nodes that are absent from the
> persisted sample.**"

Task 185's Why (`plan/tasks/185-...md:20-23`):

> "Task 183 then built two distinct alternative 4,096-row heads, but **exact
> scoring returned the same ordered top-32 seeds and the same 0.9625 recall.**"

Paper §3 states the cause directly:

> "In order to ensure that the entire graph is reachable, we build the head index
> from the union of the top layers of **each partition's** graph, **rather than
> the top layers of the stitched-together graph**."

ecaz builds from the stitched global graph: `shard_build.rs:587-589` returns a
global-space graph; `ambuild.rs:122-169` samples it from a single global medoid.
Default `build_shards = 1` (`mod.rs:247`), so the common case has no partitions
at all. The paper explicitly warns this construction does not ensure
reachability.

| Paper element | ecaz today | Citation |
| --- | --- | --- |
| §2.2 BFS over **top layers** to collect C | flat BFS prefix from one global medoid (single-layer graph); under the promoted policy, no traversal at all | `head_sample.rs:241-289`; `head_sample.rs:452-537` |
| §2.2 head is a conventional **ANN index** | Vamana graph is built, persisted, digested, validated, loaded — then **never traversed**. The promoted policy brute-forces 4,096 full-precision inner products plus a full 4,096-element sort per query, single-threaded on the coordinator, to yield 32 seeds | `head_sample.rs:1048-1050`, `1130-1165` |
| §2.2 head is **sharded** | coordinator-local only; `DistannPhysicalHeadIndex` appears in no owner, handoff, or remote-transport path | `generation_read.rs:2318` |
| §4.1 fix for CPU-bound head = **more replicas** | no head replica concept; only a thread-local 2-entry backend cache | `generation_read.rs:261-277` |
| §3 union of **each partition's** top layers | stitched global graph, before hash placement | `shard_build.rs:587-589` |

The paper's two structural remedies are logged in the roadmap as `HEAD-11`
"unmeasured" and `HEAD-12` "deferred", and are **outside the scope of both Task
185 and Task 186**. The 180->186 program has been iterating selection objectives
over a head whose construction method is the suspect.

Two further findings:

- **The promoted policy was defined as diagnostic.** `plan/tasks/181-...md:108-110`
  defines `training_landmarks` as "a **diagnostic** policy that greedily
  covers/frequency-ranks owner-oracle seeds from the disjoint training queries".
  It ranks nodes by how often they appear in the top-32 for 200 *training
  queries* (`head_sample.rs:452-497`). Task 182 promoted it to production.
- **Spec/code divergence.** `FR-080:22-27` specifies per-shard-medoid BFS with
  hop radius and per-shard union — **no code implements it**. `FR-080:44-52`
  specifies a 2-entry LRU keyed on `(index_oid, uuid, build_id, fingerprint)`;
  `head_cache.rs:75-106` is an unbounded `HashMap` keyed on `index_oid` alone.

## Defect 4 — The replica abandoned the distributed premise

Tasks 198/199 built and promoted a coordinator traversal replica holding
`(vec_id, owner_ordinal, graph_record, exact_vector)` for every vec_id from every
owner — full-precision f32 (`traversal_replica.rs:275`: `dimensions * size_of::<f32>()`)
plus the complete graph record. Measured 1,659,518,976 bytes at 100k
(~16.6 KB/vector), linear in N, on one node. With a Ready replica, traversal is
entirely coordinator-local and owners are contacted only for payload columns.

**Three rules already prohibited it. None is cited in any 198/199 packet.**

- `NFR-017:38` — the latency/recall gate itself: "A replicated full index with
  serving-ownership filtering or tombstoned non-owner records is an optional
  control lane and **cannot satisfy this NFR**."
- `NFR-018:35` — "not a valid NFR-018 distributed measurement lane", with metric
  row `non-owner graph records in the measured lane | 0 | 0`.
- `FR-078:491-501` — "A coordinator inside the serving roster SHALL store only
  its own graph-node shard ... A replicated full index hidden behind
  serving-ownership filtering SHALL NOT satisfy this requirement."

`FR-084` was written as a carve-out that evades FR-078 textually: the replica is
a separate derived relation with its own columns, so it is not literally "a
graph-node shard". `ADR-086` cites **no NFR at all**, while its own Consequences
(line 165) acknowledge "linear per-coordinator amplification".

**Provenance of the rule that was broken.** `git log -S` locates the NFR-018
exclusion clause in commit `32b9b43fb` (2026-07-10, the Task 179 spec commit),
which also touches `plan/tasks/172-...md` in the same commit. It was written as
the direct response to **Task 172 being SHELVED for exactly this defect** — a
fixture that replicated the full graph per node, making its storage/latency/recall
numbers artifacts rather than a gate. Tasks 198/199 shipped the same defect as a
feature thirteen days later.

**The principle predates ec_distann.** `ADR-067:47-51` rejected the SPIRE
CustomScan design because:

> "**Storage does not scale out.** ... Aggregate dataset size is bounded by the
> coordinator's single-machine storage capacity. The 'distributed' property is
> limited to compute parallelism on a shared dataset, not storage scale-out."

and at `:196`, "at the cost of **the most important architectural property of a
distributed vector search system**." That rationale lived only in an ADR's
rejection note and was never lifted into a requirement — which is why nothing
caught it the second time. `NFR-021`, landed with this packet, fixes that.

**Narrowing failure.** Task 190 listed family 1 as
"`ARCH-01`, `ARCH-02`, `TRAV-28`--`TRAV-30`", then carried only
`ARCH-02`/`TRAV-28` into the final comparison. **`TRAV-30` — "routing-only
gateway copies without full graph replication", the one candidate preserving
sharding — was dropped at the narrowing step.** Task 190 then set "a hard storage
budget of no more than one additional physical generation per coordinator
(2,496,626,688 bytes at 100k)" without asking whether the result remains
distributed. `TRAV-28` itself reads "Replicated coordinator **top-layer** graph";
it shipped as a full-graph replica without the ledger entry changing.

### 4b. The storage evidence could not have detected the cost

This is an independent evidence failure, sufficient on its own to reopen the
Task 199 promotion.

The suite's storage step computes four scalars **before** the variant loop
(`crates/ecaz-cli/src/commands/dev/distann_multicluster.rs:5153-5160`) and
reprints them unchanged **inside** it (`:5209-5212`):

```
5153	    let physical_generation_bytes = published
5154	        .iter()
5155	        .map(|row| row.graph_bytes + row.row_bytes + row.directory_bytes + row.control_bytes)
5156	        .sum::<i64>();
```

The owner and replica storage rows are therefore byte-identical **by
construction, not by measurement** —
`reviews/task-198/004-isolated-100k/artifacts/run/replica-isolated-ab-100k/distann-multinode-summary.log:163,166`
differ only in `variant=` and `traversal_replica=`, both carrying
`physical_generation_bytes=2496659456`. Same in `results.jsonl` lines 158/161, and
in task-199 `results.jsonl` lines 53/56.

The replica's bytes exist only in a **log-only** metric,
`physical_benchmark_traversal_replica ... relation_bytes=1659518976`
(`.../distann-multinode-summary.log:13`). `grep -c relation_bytes` returns **0**
for every `results.jsonl` in all three packets. Additionally, the topology rows
feeding `physical_generation_bytes` are captured at `phase=ready`/`phase=published`
— **before** the replica is built — so the replica could not enter the numerator
even in principle.

`cluster_index_space_amplification`, the real NFR-018 ratio emitter at
`distann_multicluster.rs:7419-7482`, exists and **ran for Tasks 172 and 197**. It
was **not run for 198/199**. `NFR-018:66` requires "the ratio row appears in the
packet manifest per scale"; it appears in none of the three. Two mutually
inconsistent hand-computed ratios stand in for it: 66.5% in Task 198
(`005-.../artifacts/manifest.md:67-71`, denominator 2,496,659,456) and 52.0% in
Task 199 (`003-.../artifacts/manifest.md:22`, denominator 3,188,056,064).

`grep -rn "NFR-018\|space amplification\|non-owner graph\|storage budget"` across
all three packets returns **zero matches**.

**A reviewer caught it before promotion.**
`reviews/task-199/003-release-matrix-and-decision/feedback/2026-07-25-01-reviewer.md:195-199`:

> "That is true and it is also the wrong sentence. `physical_benchmark_storage`
> never reports the replica image, so 'identical' is a statement about what the
> storage step does *not* measure, presented in the storage row of a
> promote/don't-promote table. A reader deciding on this packet would conclude
> the replica costs nothing. It costs 1.66 GB and ~1.94 GB of WAL at 100k."

It was answered with a prose note in the manifest and promoted. This is a process
finding as well as an evidence finding: the gate existed, the reviewer fired, and
the decision proceeded.

---

## Decision matrix

Verdicts: **SOUND** — disposition stands. **SUSPECT** — finding may stand but the
evidence cannot support the disposition as written; re-measure before relying on
it. **INVALID** — disposition rests on an inadmissible control or unmeasurable
evidence. **PENDING** — not yet audited to citation standard; the specific
outstanding check is named.

| Task | Disposition of record | T1 | T2 | T3 | T4 | Verdict | Basis / required action |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 161 | spec authoring, in progress | — | — | — | ? | **PENDING** | Verify FR-080/FR-084 spec-vs-code divergences listed in Defect 3 are the complete set |
| 162 | done, banded M0 exit | ok | ok | ok | ok | **SOUND (finding) / NOT ACTIONED** | The G0 measurement is correct and was never applied. Its "multinode wants >=32" conclusion is the strongest unactioned result in the program |
| 163 | partial, D8 review requested | — | — | — | ? | **PENDING** | Verify stitch output feeds head construction per §3 |
| 164 | partial — "replicated-serving control only" | **fail** | **fail** | fail | ? | **INVALID (self-declared)** | Status line already admits the control. Nothing measured on it can gate |
| 165 | partial — "replicated-serving control only" | **fail** | **fail** | fail | ? | **INVALID (self-declared)** | As 164 |
| 166 | measured — "single-instance control only" | **fail** | ok | fail | ? | **INVALID (self-declared)** | Single-instance control is exactly what NFR-022 now forbids as a decider |
| 167 | partial, needs physical adaptation | — | — | — | ? | **PENDING** | DML path; no disposition to invalidate |
| 172 | SHELVED — replicated fixture | n/a | n/a | n/a | ok | **SOUND** | The shelving is correct and is the precedent that produced NFR-018:35. Re-check whether FR-078 sharding now permits unshelving |
| 179 | physical hash-shard generations | ok | ok | fail | ? | **SOUND (impl) / SUSPECT (BW16/H25)** | Packet 066 tested at **fixed BW x H = 400**; it cannot speak to raising the budget. Reviewer's `2026-07-13-01` BW=4 finding never actioned |
| 180 | completed — measured negative, width/seed | ok | ok | **fail** | ok | **SUSPECT** | `NEG-01` swept seeds {32,64,128} at BW=4, where the beam pops 4/round and extra seeds are structurally unusable. Valid for BW=4 only |
| 181 | completed — GO | ok | ok | fail | ok | **SOUND** | Strongest result in the program: membership, not search, bounds recall. Points directly at §3, which the follow-on tasks did not pursue |
| 182 | completed — PROMOTE trained policy | ok | ok | fail | ok | **SUSPECT** | Promoted a policy Task 181 Phase 2 defined as **diagnostic**, selected on training-query frequency. Recall gain is real; the selection rationale needs re-derivation |
| 183 | complete — STOP | ok | ok | fail | ? | **PENDING** | Verify the two alternative heads were built from the stitched graph (if so, `NEG-06` is qualified the same way as `NEG-01`) |
| 184 | complete — PROMOTE lazy10 | ok | ok | n/a | ? | **SOUND (provisional)** | Owner-arm control, bounded window of 10, owner-side. Confirm no storage claim rests on the arm-blind step |
| 185 | proposed | ok | ok | fail | n/a | **SCOPE GAP** | Holds cap/exact-scan/32-seeds fixed and excludes `HEAD-11`/`HEAD-12` — the paper's actual remedies. Re-scope before executing |
| 186 | proposed, conditional on 185 | ok | ? | fail | n/a | **SCOPE GAP + NFR-021 SCREEN** | "Larger head" must state a bound; cap 8,192/16,384 is fine, `C = f(N)` requires sharding |
| 187 | complete — STOP, no candidate | ok | ok | fail | ok | **SOUND (attribution) / SUSPECT (STOP)** | Attribution is valid; the STOP inherits Defect 2 |
| 188 | complete — accept BW8, no production change | ok | ok | **fail** | ? | **SUSPECT** | BW=8 without pushdown, paired with an experimental 16,384-landmark head. A 2x step against a 32x design |
| 189 | proposed, dormant | — | — | — | n/a | **PENDING** | No disposition |
| 190 | complete — select traversal replica | **fail** | **fail** | fail | ok | **INVALID** | Dropped `TRAV-30` at narrowing; budgeted an O(N)-per-coordinator structure without asking whether the result stays distributed. Root decision of Defect 4 |
| 191 | complete — PROMOTE production lazy10 | ok | ok | n/a | ? | **SOUND (provisional)** | As 184 |
| 192 | complete — PROMOTE schema cache | ok | ok | n/a | ? | **SOUND (provisional)** | Owner-side, bounded by relation/projection count |
| 193 | complete — STOP | ok | ok | n/a | ok | **SOUND** | Negative result, no production change |
| 194 | complete — STOP `TRAV-14`/`TRAV-15` | ok | ok | **fail** | ok | **INVALID (STOP) / SOUND (counters)** | The nine-way TRAV-01 attribution is excellent and stands. The STOP measured widening without pushdown at BW=8. Ledger rows voided |
| 195 | complete — PROMOTE owner schema cache | ok | ok | n/a | ? | **SOUND (provisional)** | Owner-side, bounded |
| 196 | complete — PROMOTE prefix reuse | ok | ok | n/a | ? | **SOUND (provisional)** | Scan-local, bounded |
| 197 | complete — PROMOTE release preflight | ok | ok | n/a | ok | **SOUND** | Ran `cluster_index_space_amplification`; a positive example |
| 198 | complete — PROMOTE to 199 | **fail** | **fail** | fail | **fail** | **INVALID** | Defect 4 + 4b. Any one of T1/T2/T4 suffices |
| 199 | complete — outside-reviewed PROMOTE | **fail** | **fail** | fail | **fail** | **INVALID** | Defect 4 + 4b; reviewer objection recorded and overridden |
| 200 | complete — memory retention fix | ok | ok | n/a | ok | **SOUND** | Benchmark-only; did not alter the production read path |
| 201 (distann) | proposed | **fail** | **fail** | fail | n/a | **INVALID BY INHERITANCE** | Frozen control (`:34`) *contains* the replica; `:43-44`/`:113` forbid replica questions entering the screen, so it cannot surface this. Unstarted — cheap to re-scope now |
| 202 | proposed — portability gate | ok | ok | n/a | n/a | **SOUND** | Cross-ISA identity; orthogonal |

### Cross-cutting

**Arm-blind storage claims.** Because `physical_benchmark_storage` is computed
before the arm loop, every packet asserting "storage identical/unchanged between
arms" asserted something the metric cannot express. This is not confined to
198/199 and is the reason several rows above carry `T4 = ?`. The full sweep is
the outstanding work item for packet 002.

**Missing NFR-018 ratio row.** Required per scale by `NFR-018:66`. Present in
Tasks 172 and 197; absent from 198/199. The remaining gate packets need the same
check.

## Recommendations

1. **Re-open 190, 198, 199** and the ec_distann 201's frozen control. Restore the
   owner arm (18.3/20.4/19.9 ms) as the program's latency baseline — done in the
   ledger with this packet.
2. **Void `TRAV-14`/`TRAV-15`; qualify `NEG-01` and probably `NEG-06`.** They
   measured the wrong regime or lacked the enabling mechanism.
3. **Re-scope Tasks 185/186** to include `HEAD-11`/`HEAD-12` — the paper's actual
   answer to the membership bound that Task 181 measured.
4. **Sequence the corrective program pushdown -> regime -> head.** Neither the
   wide-beam nor the seed-width question is answerable until Algorithm 1's
   pushdown exists.
5. **Fix the storage step before any further gate run** (`distann_multicluster.rs:5153-5212`),
   or every future arm comparison inherits the same blindness.
6. **Do not delete FR-084 or the replica.** Its disposition — bounded top-layer
   re-scope, or explicitly waived non-conforming accelerator — is an operator
   decision, and the corrective program may remove its motivation entirely.

## What this packet does not do

No `Status:` line in `plan/tasks/` is changed. No `src/**` is changed. No
benchmark was run; every citation is a re-read of committed evidence. Verdicts
are recommendations to the operator, not closures.

## Artifacts

See `artifacts/manifest.md` for the citation index and the commands used to
establish provenance.
