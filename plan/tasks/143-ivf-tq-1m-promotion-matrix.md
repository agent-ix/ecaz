# Task 143: IVF TQ 1m promotion matrix (layout × scorer defaults)

Status: **complete — defaults landed** (2026-07-03, operator-approved). Owner: Codex. Priority: P2

Outcome: both defaults flipped at `815518d82` and confirmed on the pure
default path (`reviews/task-143/002-default-flip/`): out-of-the-box ec_ivf
TurboQuant now builds dense posting blocks and scores with the int8/SDOT
kernel — 100k 3.16→1.75 ms, 1m 12.1→7.76 ms vs the old defaults at
recall within noise; nprobe 40 offers +recall AND −latency vs the old
operating point. Graviton remains the recorded cross-lane follow-up.

Evidence: `reviews/task-143/001-promotion-matrix/` — old default (row+lut)
→ proposed (dense+int8) at nprobe 32: 100k 3.16→1.75 ms (−44.6%), 1m
12.1→7.76 ms (−35.9%), recall dips ≤0.42 pp (in noise), dense recall
byte-identical to row at all 32 recall cells, storage −9.6/−9.9%. Both
default flips recommended; awaiting reviewer approval to land them.
Gated on: (a) Tasks 141 (SDOT) and 142 (drain policy) landing first — the
1m matrix should measure the final candidates once; and (b) the
`task-125-tq-scorer-optimization` + `task-136-rank1-scorer` stack being
reviewed and landed on `main` — the default-flip and promotion decisions
must measure what ships, and the expensive 1m run should not precede
review-driven changes underneath (branch decision, operator 2026-07-02;
main had not diverged from the stack at that point, so landing is a clean
linear merge). 1m benches on the m5-local lane are authorized (operator,
2026-07-02).

## Why

Two default decisions now have 10k/50k/100k evidence but await scale
confirmation:

- `ec_ivf.turboquant_scorer=int8_approx` (Task 136): −14/−14/−16.5% mean
  latency at recall within noise (`reviews/task-136/001-int8-approx-ivf-scorer/`).
- `dense_posting_blocks=1` (Task 135 packet 002 / Task 111a family):
  posting_visit−flush −24/−29%, storage −8/−10%, recall byte-identical
  (`reviews/task-135/002-dense-layout-ab/`). The Task 111a closeout kept the
  reloption gated pending 1m evidence, and the dense win is expected to GROW
  at 1m: the ~850 MB 1m index will not be buffer-resident (shared_buffers
  128 MB), and Task 135 packet 001 showed page-access cost is ~4× higher on
  buffer-pressured tables — dense's ~10× fewer posting pages should matter
  most exactly there.

## Prerequisite: stage the 1m fixture

- Generate `ec_real_1m_{corpus,queries}.tsv` + `_manifest.json` into
  `data/staged-current/` via `ecaz corpus prepare` from the local dbpedia
  parquet base (`data/task31_m5_dbpedia_fetch/data/`, 26 shards, 8.9 GB).
  Expect a ~21 GB corpus TSV; disk has headroom (~479 GB free at authoring).
- Record the prepare command, prefix, scale, and TSV SHAs in the packet
  `manifest.md` per NFR-007. Never commit the TSVs.

## Matrix

{row, dense_posting_blocks=1} × {lut, int8_approx} at **100k + 1m**:

- recall + latency + storage per cell, `ivf_stage_counters` +
  `task87_candidate_batch_counters` on.
- nprobe [32] fixed point for stage attribution, plus the registered
  `ec_ivf` default sweep `[8,16,24,32,48,64]` for the headline
  recall/latency cells (standard-sweep rule; state any deviation in the
  manifest).
- Same-session pairing per axis; verify `ecaz_build_git_sha()` before and
  after; no dylib installs mid-run (including `cargo test`, which installs
  a debug dylib).
- Extra cell: nprobe/recall trade under int8_approx — nprobe 40 (≈ the old
  latency budget) to measure whether 100k/1m recall rises ~1–2 pp "for
  free".

## Decisions the packet must recommend (with evidence)

1. Flip `ec_ivf.turboquant_scorer` default to `int8_approx`, or keep
   opt-in (Task 136 follow-through).
2. Promote `dense_posting_blocks` to default for TurboQuant IVF, or keep
   gated — feeds the Task 111a-family promotion decision. Graviton/AWS
   remains the recorded open lane for both (not a blocker for the m5-local
   recommendation; note it explicitly).

## Out of Scope (hard)

- No new code paths; this is a measurement + decision task. Any fix it
  motivates becomes its own task.
- No AWS/Graviton execution unless the operator opens the lane.

## Gate / Exit Criteria

- 1m fixture staged and recorded per NFR-007.
- Full 2×2 × {100k, 1m} matrix with recall/latency/storage + counters in
  one packet, and an explicit promote / keep-gated recommendation for each
  of the two defaults.
