# Task 76: SPIRE Recall — Default Pareto Point

Status: complete (2026-05-31, closeout `reviews/task-76/002-closeout/`)
Owner: coder (to be assigned). One coder, one branch.
Priority: 2 (recall-side complement to Task 75; gated on Task 75 Phase 1)

## Why

Task 73 closed with two facts that don't sit easily together:

- The stock 100k SPIRE setting (`tg16 b0 nprobe=16`) hits recall@10
  `0.8525`, which is well below cross-AM bars (IVF `0.9980` at
  nprobe 96, HNSW `0.9775` at ef=400).
- The known recall-recovery setting (`tg128 b0 nprobe=96+`) hits
  recall@10 `0.9975-1.0000` but at `4-7x` IVF latency.

Task 73 shelved a default change because the high-recall point is
slow. `plan/design/spire-quality-defaults-followup.md` records
that as a product/defaults question. This task is the
**investigation** that would feed that product decision: is there
a useful Pareto point between `0.8525 / 14 ms` and
`1.0000 / 96 ms` that today's defaults miss?

The Task 73 sweep already hints at one. On M5 100k:

| tg | b | nprobe | recall@10 | p50 |
| ---: | ---: | ---: | ---: | ---: |
| 16 | 0 | 16 | 0.8525 | 13.5 ms |
| 32 | 0 | 32 | 0.9310 | 27.1 ms |
| 64 | 0 | 64 | 0.9825 | 51.2 ms |
| 128 | 0 | 96 | 0.9975 | 75.8 ms |

The `tg32 nprobe=32` row (`0.9310 / 27.1 ms`) is a roughly `2x`
latency cost for `+7.85 pp` recall — a much better tradeoff than
the next jump. Whether that's the right product default depends
on three open questions this task answers:

- Does the Pareto shape hold at 10k and 1M, not just 100k?
- Does it hold across hosts (Intel desktop, AWS Graviton),
  not just M5?
- Does it hold across real-corpus shapes other than DBpedia, if
  any are available?

## Non-Goals

- Do not change SPIRE on-disk format or recursion semantics.
- Do not change routing aggression as a *code* change here — Task
  75 owns that axis. This task is defaults-only.
- Do not pursue cross-AM recall claims as a publishable result.
- Do not pursue adaptive runtime defaults (row-count-aware
  selection) as a P0 — that's a follow-on if the Pareto shape
  varies by corpus size.

## Phase 1 — Pareto Curve at 10k / 100k / 1M (gating)

Land one measurement packet on the Intel desktop using
`ecaz bench suite`. Required contents:

- At each of 10k / 100k / 1M (1M is gated on whether a 1M real
  fixture exists locally; if not, document and skip):
  - Sweep `top_graph_search_list_size` across `16, 32, 64, 96, 128`
    with `boundary_replica_count=0`.
  - For each setting, sweep `nprobe` at and below
    `top_graph_search_list_size`.
  - Capture recall@10, p50, p95, p99, mean.
- IVF and HNSW controls at each corpus size with matched recall@k.
- Identify candidate Pareto points and tabulate them as
  `(recall, p50, p95)` rows.
- Compare to Task 75 Phase 1 candidate-envelope evidence if
  Task 75 has landed: the per-setting `candidate_sum` explains
  *why* the Pareto curve looks the way it does.

Phase 1 closes when the packet has:

- a Pareto table per corpus size,
- a recommended default setting (or "no setting beats the
  current default once tail and small-corpus behavior are
  considered"),
- explicit cross-corpus tradeoff notes (where the recommended
  point regresses).

## Phase 2 — Default Change (only if Phase 1 surfaces a clear win)

P0 slices land one at a time. Likely shapes:

1. **Change `top_graph_search_list_size` and `nprobe` defaults**
   in `src/am/ec_spire/routine.rs` (or wherever reloption
   defaults live) to the Phase 1 recommendation, with the
   defaults task wired so existing indexes are not silently
   re-tuned at read time.
2. **Add a quality-preset reloption** (e.g.
   `spire_quality=balanced|fast|high`) that maps to vetted
   `(tg, nprobe)` pairs, keeping the current default unchanged.
3. **Adaptive default** keyed on corpus size — only if Phase 1
   shows the right Pareto point shifts with corpus size and a
   simple selection rule fits the evidence.

Any slice must:

- preserve recall on every measured corpus size (no regression
  on 10k while improving 100k),
- preserve or improve p95 on the affected setting,
- be reviewed against the active SPIRE recursion contract (the
  recursion owner from Task 30 is no longer active, so this
  needs an explicit owner check at slice time),
- include before/after measurement on at least Intel desktop and
  AWS Graviton.

## Exit Criteria

- Phase 1 packet landed with reviewer-approved Pareto table and
  decision.
- Either a Phase 2 default change landed with measured cross-host
  evidence, *or* the closeout records why the current default
  remains the best available choice.
- Recall on 10k preserved per Task 68/73 documented points.
- `cargo clippy --all-targets --no-default-features --features
  pg18 -- -D warnings` clean.
- Closeout packet flips `plan/tasks/76-…md` status to `complete`
  and either supersedes
  `plan/design/spire-quality-defaults-followup.md` or amends it.

## Coordination

- **Task 73** is closed; this task implements the defaults
  follow-on it deferred.
- **Task 75** runs in parallel on the latency axis. If Task 75
  lands a routing-tightening slice during this task's Phase 1,
  rerun the affected Pareto rows before settling on a default.
- **Task 72 (SPIRE parallel build)** is independent.
- **Task 30** phase tree is closed — flag any
  recursion-semantics question at slice time rather than
  proceeding.

## Stop Conditions

- Stop Phase 2 if Phase 1 shows the current default is
  Pareto-optimal once tail latency and cross-corpus behavior are
  considered.
- Stop Phase 2 if the recommended default change requires
  recursion-semantics changes (file as a new task instead).
- Stop if 1M fixture is not available locally and the cross-size
  evidence is therefore restricted to 10k/100k; surface the gap
  in the closeout rather than ship a 1M-uninformed default.
