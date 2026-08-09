# Task 218: ec_distann Owner-Side Materialization Latency (MAT-16 / MAT-21 / MAT-22)

Status: **complete — review-closed ACCEPT, MAT-21 STOP** (2026-08-08; packets
001+002, `reviews/task-218/002-isolated-candidate/feedback/2026-08-08-01-reviewer.md`).
P1 measured the production lazy-10 denominator (owner endpoint 9.10 ms/scan of
an 18.83 ms scan); P2's typed `tid[]` locator was neutral end-to-end, so no
release matrix. Carried: state the MAT-16/MAT-22 disposition (retired / carried
/ open) and reconcile the roadmap MAT-16/21/22 candidate rows — this STOP
retires the locator hypothesis, not the ~8.5 ms/scan owner payload SQL stage.
Priority: P1 latency.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`.
Origin: Task 216 carry-in.

This is a **new task**, not a reopening of Task 216. Task 216 closed the
*coordinator-side* family; it explicitly did **not** retire the owner-side
candidates, and its decision text says so.

## Why

Task 216's isolated screen established a 0.19% addressable ceiling for
coordinator decode (0.076 ms against a 40.60 ms scan) and retired
MAT-12/13/14/15 on it. Its own decision then bounds that screen:

> "MAT-16 and MAT-21 remain owner-side stage candidates and are not retired by
> the coordinator ceiling; MAT-22 remains an owner expansion/wire candidate."

The stage that survives is the dominant one. At 100k the attribution measured
`remote_materialize` at **25.96 ms** of a 37.21 ms `custom_scan_total`, with
`materialize_owner_payload_sql_work` and `materialize_owner_endpoint_work` the
constituents. Nothing else in the scan is close: traversal 8.21 ms, coordinator
decode 0.075 ms, owner response encode 0.002 ms.

## The denominator problem this task must fix first

The Task 216 reviewer's P2 finding (`005` feedback) is a hard precondition:

> "`materialize_owner_payload_sql_work` at ~39 ms/scan was measured while
> materializing 31.34 payloads per scan; production lazy-10 materializes a
> bounded window, and the counters show only 6.66 rows reaching the executor.
> The owner-side stage that MAT-16/21/22 target is therefore materially smaller
> in the production profile than the attribution number implies. ... their
> addressable budget is currently unmeasured, not ~39 ms."

That arm ran `materialization_batch_size=0` — the eager control. **Phase 1 of
this task re-measures the owner-side stage on the production lazy-10 profile**,
and the resulting number, not the eager one, is the budget every candidate is
screened against.

## Entry gate

Task 217's same-generation lane is landed and its A/A run reproduces. Without
it, no candidate here can produce an attributable recall or ordering claim.

## Phases

### P1 — Production-profile attribution

Re-run the 100k owner-side attribution with production lazy-10
(`materialization_batch_size=10`), on a featureless normal-release build where
possible, and record the true addressable budget for owner payload SQL,
endpoint work, and locator formatting. **If the budget is small, this task may
close STOP here** — that is a legitimate outcome and cheaper than three A/Bs.

### P2 — At most one candidate, pre-registered

Advance **one** of:

- **MAT-16** — avoid PostgreSQL array construction for each payload row.
- **MAT-21** — replace textual `ctid` formatting with typed/binary locators.
- **MAT-22** — return the row-tier locator with expanded candidates, removing a
  lookup from the materialization path.

Selection is pre-registered against the P1 numbers, not chosen after seeing
arm results.

### P3 — Release matrix, only if P2 is useful end to end

10k / 50k / 100k recall + latency + storage on the standard lane, sharded owner
control (NFR-022), NFR-021 conformance on every arm. A win here is still not a
default change; productionization is a separately numbered task.

## Non-goals

- Coordinator-side payload work (closed by Task 216's ceiling).
- Beam width, hop rounds, `L`, `top_k`, seed count (205/206/215/219).
- Head construction or selection (207/185/186).
- Changing the shipped default from within this task.

## Required review packets

1. `reviews/task-218/001-production-profile-attribution/`
2. `reviews/task-218/002-isolated-candidate/`
3. `reviews/task-218/003-release-matrix-and-decision/` (only if 002 is useful)

## References

- `reviews/task-216/001-attribution/artifacts/attribution-disposition.md`
- `reviews/task-216/005-isolated-candidate-correction/feedback/2026-08-07-01-reviewer.md`
- Roadmap rows MAT-16, MAT-21, MAT-22
