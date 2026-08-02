# Task 211: ec_distann Head Scaling Law

Status: **complete** (2026-08-01). Priority: P1.

P0 spec landed as
`spec/functional/distann/read/FR-088-distann-head-scaling-law.md`
(hardened by the failure-domain/integrity/scope-boundary round: rate-only
precedence, `EC_HEAD_SIZING` validation, build-options v3 attestation
carrier, pinned f64 arithmetic); packet
`reviews/task-211/001-head-scaling-law-spec/` open. P1 build-side law and the
P2 staged evidence gate are complete in
`reviews/task-211/002-head-scaling-law-implementation/`. The measured 0.02
candidate remains an explicit opt-in law; the shipped default remains the
fixed 4096 cap because the corrected A/B evidence did not show a consistent
win at all three staged scales.

Entry gate: Task 210 merged (sharded, membership-only head is the shipped
default — `reviews/task-210/006-zero-byte-head/`). Tasks 212/213 build on
this task's spec but only their final sizing sweeps depend on its outcome.

## Why

The head is currently a fixed cap (`head_index_cap`, default 4096) — constant
in `N`. The head's job is seed quality: with `C` fixed and `N` growing, each
landmark covers `N/C` vectors, seeds land farther from the query's true
neighborhood, and the deficit is paid in extra beam rounds — and in this
architecture a hop is a full owner fan-out round trip. The reference design's
head is a scaled-down index over a **sample** of the corpus: a sampling rate,
not a fixed cap. The constant cap is an artifact of the current benchmark
regime, not a design endpoint; it is also exactly the geometry (`fixed C`,
growing `N`) that made the old constant-`C` exemption load-bearing in the
wrong direction before Task 210.

Now that the head is sharded and membership-only (zero coordinator bytes), a
growing head costs the coordinator nothing — growth lands on the owners,
where it belongs. That removes the old reason to keep it small.

## Goal

Head size is a **sampling-rate law** applied at epoch build, not a fixed cap.
A default law is chosen from measured evidence and shipped as the default.

## Phases

- **P0 — spec first.** Update the spec with `/specify` (FR for head sizing:
  the law, its bounds, where it is applied, how the manifest attests it) and
  validate with `/spec-review` before implementation. The head-sizing text
  currently reads as a fixed build option; the requirement should express
  rate-plus-floor/ceiling semantics.
- **P1 — build-side law.** `head_index_cap` becomes derived:
  `C = clamp(rate × N, floor, ceiling)` (exact form per the spec), resolved at
  epoch build (T2) from the build's record count, attested in the epoch
  manifest. Explicit cap remains available as an override for fixtures.
- **P2 — sweep and pick.** Sweep sampling rate at **10k/50k/100k** (recall +
  latency + storage + `traversal_hop_rounds`/frontier counters per arm, via
  `ecaz bench suite`), pick the default law, land it.

**Scale bound (deliberate):** sweeps stop at 100k for now. 1M+ validation is
deferred until the build-side optimization backlog lands — the build cost of
larger heads at 1M+ is not worth paying yet, and the law's *shape* (not its
asymptote) is decidable at the staged scales. Re-validating the chosen rate at
1M+ is an explicit follow-up gate before any 10M+ claim.

## Benchmark gate

Standard 10/50/100k A/B per candidate rate against the current fixed-cap
default, one rate per arm (no stacking), `ecaz bench suite` config committed
in the owning packet. Hop-count deltas must be reported alongside
recall/latency — hop reduction is the mechanism, so an arm that improves
latency without moving hops is flagged, not celebrated.

## Stop conditions

None structural. If no swept rate beats the fixed cap at these scales, the
honest outcome is "law implemented, default unchanged, re-sweep at 1M+" —
the mechanism ships either way; only the default stays put.
