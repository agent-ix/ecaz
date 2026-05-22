# Task 47 / 008 — Task 47 Closeout

## Goal

Walk every Task 47 exit criterion in
`plan/tasks/47-recall-and-cost-model-gates.md` against the live
repo and cite the artifacts that satisfy each, mirroring the
`reviews/task-39/039-task39-closeout/` pattern.

The 007 packet's reviewer feedback (2026-05-21-01) wrote:
"Cascade closeout finally feasible across both Task 39 (after
053 wire-format audit) AND Task 47 (after this packet lands)."
This packet is the requested Task 47 close.

## Exit Criteria Audit

`plan/tasks/47-recall-and-cost-model-gates.md` lists four exit
criteria. Each is evaluated below against the live repo.

### 1. `make recall-gate` runs in PR-CI with documented per-AM floors

**Met.**

- `.github/workflows/ci.yml::recall-cost-gates` runs `make
  recall-gate` against the small gate fixture loaded earlier in
  the same job (HNSW + IVF + DiskANN profiles at 512 vectors /
  32 dims).
- `docs/recall-floors.md` documents the per-AM floors and the
  PR vs nightly cadence per gate.
- `fixtures/gates/recall-gate-small.json` has the threshold rows
  consumed by the gate.

### 2. `make cost-gate` runs in PR-CI with a committed baseline that updates via explicit packet

**Met.**

- `.github/workflows/ci.yml::recall-cost-gates` runs `make
  cost-gate` after the recall and cross-AM steps.
- `fixtures/cost-queries/baseline.json` is the committed baseline.
- `fixtures/cost-queries/README.md` documents the update path:
  baseline drift requires `--accept-drift` + an explicit Task 47
  packet with the raw planner-cost logs.

### 3. `docs/recall-floors.md` and `fixtures/cost-queries/` are authoritative

**Met.**

- `docs/recall-floors.md` is the single-source-of-truth for gate
  cadence, fixture contracts, and per-AM floor values. Updated
  via packet 007 (cross-am-gate row) to drop the
  "PR candidate, report-first" qualifier; all three gates now
  carry PR cadence unambiguously.
- `fixtures/cost-queries/` contains the baseline JSON plus the
  README pointing at the regeneration command and the
  "no merge without a packet" contract.

### 4. The existing `recall_integration` tests are either retired or marked redundant

**Met.**

`tests/recall_integration.rs` opening doc-comment now reads:

> Task 47 live AM gating now lives in `make recall-gate`; this
> ignored harness remains only as a pure-Rust quantizer
> benchmark.

The test runs only with `--ignored` behind the `bench` feature.
It is no longer the source-of-truth for AM recall — the gate
suites are.

## Cross-AM gate completes the core enforcement set

Packet 007 promoted `make cross-am-gate` from "PR candidate,
report-first" into the PR-CI job alongside `recall-gate` and
`cost-gate`. After 007, the PR-blocking gate trio is exactly
the three correctness gates the Task 47 plan named in its
Approach section:

- `recall-gate` — per-AM recall vs brute-force differential.
- `cross-am-gate` — HNSW/IVF/DiskANN top-k consistency.
- `cost-gate` — planner-cost-model drift bands.

## Non-blocking follow-ups noted by the 007 reviewer

The 007 feedback flagged three follow-ups that are out-of-scope
for Task 47 closure but worth a future packet:

1. Tighten the cross-AM Jaccard floor (currently `0.1`, very
   loose) once enough real-corpus baseline data has accrued.
2. Add a `docs/recall-floors.md` note explaining the
   "tighten-later" framing on the Jaccard floor.
3. Add per-query reporting + ratchet workflow as a next-slice
   packet.

None of these are exit criteria for Task 47 itself; they are
optional refinements to the now-landed gate set.

## Plan status

After this packet,
`plan/tasks/47-recall-and-cost-model-gates.md::Status` should
flip from `**proposed**` to `complete` (matching the project's
convention as used on tasks 01/02/04 and now Task 39).

## Validation

No artifacts needed for this audit packet — each exit-criterion
citation points at a path that is already committed and
inspectable in the live repo (and reviewer-accepted in the
upstream packets 001-007).

## Reviewer Direction

- Confirm all four exit criteria are met (this packet's audit
  + the 007 acceptance lines).
- Confirm the plan status flip is appropriate.
- The three non-blocking follow-ups listed above can become
  their own packets later if/when the team wants the
  tighter-floor / per-query / ratchet work.
