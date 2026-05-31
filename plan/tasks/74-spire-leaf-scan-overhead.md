# Task 74: SPIRE Leaf-Scan Overhead Audit

Status: complete (2026-05-31, closeout `reviews/task-74/002-closeout/`)
Owner: coder (to be assigned). One coder, one branch.
Priority: 2 (latency-side complement to Task 73's recall work)

## Why

SPIRE leaves are IVF-shaped partition objects. The per-leaf
scoring kernel comes from the underlying storage format
(TurboQuant / PqFastScan / RaBitQ) — shared with `ec_ivf`. But
SPIRE wraps that scan with substantial orchestration:

- recursive routing draft → leaf PID selection
- top-graph frontier traversal → candidate sourcing
- boundary-replica handling
- candidate budget enforcement (per Phase 10's bounded collection)
- multi-store / multi-NVMe access shape
- remote vs local leaf differentiation (per Phase 13d)
- per-rescan setup and snapshot management

The SPIRE-side scan code is ~4900 lines across
`src/am/ec_spire/scan/{routing,candidates,leaf_rows,relation,
snapshot,types,callbacks}.rs`. That's a lot of code path
above the actual per-leaf scoring loop.

Task 30 Phase 10 + Phase 13d closed the **architectural**
execution path (bounded candidates, AM scan shape, remote
overlap, observability). Neither task profiled the SPIRE-side
scan code for **per-query CPU overhead** that lives above
the leaf-scan kernel itself — i.e. is SPIRE's local scan
spending time on orchestration vs actual scoring?

This task answers that question with a measurement and lands
narrow optimization slices if the orchestration overhead is
material.

## Non-Goals

- Do not change SPIRE on-disk format, partition object schema,
  or wire format.
- Do not change recursion semantics. Task 30 phases own that.
- Do not reopen IVF leaf-scan kernel work. If the audit identifies
  IVF-side kernel improvement opportunities, file a separate
  IVF-scan task — this task is strictly about SPIRE-specific
  overhead above the IVF leaf.
- Do not pursue distributed / remote scan path changes. Phase 13d
  owns the remote dispatch surface; this task is local-only.
- No first-pass AWS / Graviton work — M5 local host only for the local
  leaf-scan/orchestration diagnosis. AWS profiling is only for
  distributed/remote overhead confirmation after Task 73's local quality
  gate passes.

## Phase 1 — SPIRE Leaf-Scan Profile (gating)

Land one M5-local measurement packet **before any slice**. Prefer sharing
this packet with Task 73's Phase 1 recall/latency sweep. Required
contents:

- M5 release-mode `ec_spire` scan at the specific recall/latency points
  Task 73 identifies on the same 10k + 100k fixtures Task 68 + Task 73 use:
  - current default: fast but low recall;
  - high-recall candidate: slower but quality-preserving, if one exists;
  - permissive ceiling point;
  - IVF control with partition-equivalent leaf scan.
- Profile via `samply` or `cargo flamegraph` — capture the
  per-query wall-time split across:
  - top-graph frontier traversal (`scan/routing.rs`)
  - recursive routing draft consumption
  - candidate budget enforcement (`scan/candidates.rs`)
  - leaf row materialisation (`scan/leaf_rows.rs`)
  - **actual per-leaf scoring kernel** (the IVF path)
  - heap rerank fetch + detoast
  - snapshot management (`scan/snapshot.rs`)
  - per-rescan setup (callbacks.rs)
- Compute the ratio of "time inside actual scoring" to "time in
  SPIRE orchestration" — this is the load-bearing answer.
- Compare to a control: run `ec_ivf` on a partition-equivalent
  corpus shape, profile the same per-query wall time. The
  difference is approximately the SPIRE-specific overhead.
- Do not profile an arbitrary fast/low-recall path as the only evidence;
  if Task 73 finds no credible high-recall local point, stop this task
  after the local overhead characterization and record that latency tuning
  is premature.

Phase 1 closes when the measurement packet has:
- a clean per-phase split,
- the SPIRE-vs-IVF orchestration overhead estimate,
- a ranked Phase 2 P0 slice list (or "overhead is small; close").

## Phase 2 — Slices (only if Phase 1 shows material SPIRE-side overhead)

P0 slices land one at a time. Candidates depend on what Phase 1
surfaces, but reasonable hypotheses:

1. **Per-rescan setup reuse** — if profile shows `scan_begin` /
   callback setup taking visible time per query, reuse scan
   state across rescans where snapshot/transaction semantics
   allow.
2. **Routing-draft cache** — if top-graph frontier traversal
   redoes work across queries that hit the same routing prefix,
   cache the resolved leaf-PID set.
3. **Candidate buffer reuse** — if `scan/candidates.rs`
   allocates per-query, reuse a scan-lifetime buffer.
4. **Leaf row decode pipeline** — if `leaf_rows.rs` deserialises
   per row in a hot loop, batch the deserialisation.
5. **Snapshot management** — if `snapshot.rs` does redundant
   visibility checks at the candidate layer, fold them into the
   leaf-scan pass.

Any slice must:
- preserve recall (Task 73's measured floor at the slice's tested
  fixture, within 0.5 pp),
- preserve determinism (same-query → same result),
- be measured against the Phase 1 split to show ≥ 5 % win on
  total scan wall time (else skip per per-slice cap),
- be reviewed against the SPIRE recursion correctness contract
  before merge.

## Exit Criteria

- Phase 1 measurement packet landed with reviewer-approved
  ranking.
- All Phase 1 P0 slices either landed with a measured wall-time
  win, or shelved with a recorded reason.
- Final measurement packet repeating the Phase-1 split, showing
  the per-query wall-time delta vs baseline.
- Recall floor preserved per Task 73's measured fixture floor.
- No new `unsafe { ... }` blocks. SPIRE scan code is already
  inside PG callback boundaries; this task should stay safe.
- `cargo clippy --all-targets --no-default-features --features
  pg18 -- -D warnings` clean.
- Closeout packet citing Phase 1 + Phase 2 evidence flips
  `plan/tasks/74-…md` status to `complete`.

## Coordination

- **Task 73 (SPIRE recall characterization)** is parallel.
  Task 73 owns the routing/recall axis; this task owns the
  leaf-scan-orchestration latency axis. The two could share a
  Phase 1 measurement packet if convenient.
- Task 73 is the AWS gate. This task must not advance to AWS profiling
  unless Task 73's M5-local recall ceiling passes and the remaining
  question is remote fanout/connect/transfer overhead.
- **Task 30 Phase 10 + Phase 13d** are the architectural
  predecessors. Read both before Phase 1 to avoid duplicating
  their characterisations.
- **Task 30 Phase 13d is "implementation checkpoint ready for
  review"** at the time of drafting — coordinate so this task's
  Phase 1 doesn't conflict with Phase 13d's in-flight commits on
  `scan/**`.
- If Phase 1 surfaces leaf-side (IVF) scan-kernel improvement
  opportunities, file those as a separate IVF-scan task. This
  task does not edit `src/am/ec_ivf/**`.
- Honor memory `feedback_dont_defer_safety_fixes`,
  `feedback_anti_pattern_b_unbounded_lifetime`, and
  `feedback_view_operations_not_accessors` in review.

## Stop Conditions

- Stop Phase 2 if Phase 1 shows SPIRE-vs-IVF orchestration
  overhead is below ~10 % of total scan wall time. At that
  point the per-query latency budget is dominated by the leaf
  scan kernel itself, which is out of scope for this task.
- Stop Phase 2 if Task 73's local permissive setting cannot reach the
  quality bar. Optimizing a path that is still below the recall ceiling is
  out of scope; hand off to routing/partition/codec diagnosis first.
- Stop if a slice introduces a recursion-semantics question
  Task 30 hasn't already answered. File the question to the
  active Task 30 phase and pause until resolved.
- Stop if a slice's projected speedup at 100k is below ~5 % of
  total scan wall time, unless it's a prerequisite for another
  slice.
