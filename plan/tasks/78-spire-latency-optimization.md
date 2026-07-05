# Task 78: SPIRE Latency Optimization

Status: closed - P0 slice shelved with evidence (2026-05-31)
Owner: coder (to be assigned). One coder, one branch.
Priority: 1 (required before revisiting SPIRE high-recall defaults)

## Why

Task 77's Intel-local candidate attribution packet showed that high-recall
SPIRE scan latency is dominated by candidate scoring because the scan admits
too many leaf candidates, not because row materialization or heap maintenance
is expensive:

- `tg64/nprobe64`: `10,420,357` candidates over 200 queries; scoring is about
  `82.9%` of measured candidate-path time.
- `tg96/nprobe96`: `15,506,227` candidates over 200 queries; scoring is about
  `82.1%` of measured candidate-path time.
- `tg128/nprobe128`: `20,000,000` candidates over 200 queries; scoring is about
  `83.2%` of measured candidate-path time.

The row-materialization and heap-retention slices Task 77 was created to test
are below the `10%` p50 improvement floor. Object reads are also measurable
(`17.934 ms` p50 at tg96/nprobe96), but the larger issue is the candidate
surface: at tg96/nprobe96, `15,506,227` candidates are scored over 200 queries
while only `5,000` survive to heap rerank and `2,000` rows are returned.

The next useful work is therefore SPIRE latency optimization at matched recall.
The first and primary hypothesis is that SPIRE must select better candidates
before or during scoring, preserving the high-recall floor while reducing the
number of candidates that reach the expensive scoring path. The primary target
is the RaBitQ storage format because it is the intended default direction for
this work. TurboQuant must stay in the matrix as a comparison and regression
guard, but it is not the primary implementation target.

## Scope

- Establish a RaBitQ-first SPIRE latency baseline at the same 100k high-recall
  points Task 77 measured, with TurboQuant runs retained for comparison.
- Identify where the current path admits too many candidates:
  top-graph routing, recursive leaf selection, per-leaf candidate filtering,
  scoring bounds, deduplication, or final retained-candidate sizing.
- Prototype latency slices in this priority order:
  1. candidate-selection changes that reduce scored candidates while preserving
     the Task 73/75 recall floor;
  2. RaBitQ-specific scoring/object-read improvements only after candidate
     reduction has been bounded;
  3. TurboQuant-only improvements only as comparison evidence or a regression
     guard, not as the primary deliverable.
- Account for object-read cost at those points and decide whether candidate
  reduction is enough, or whether a RaBitQ storage-format/object-layout slice is
  also needed.
- Decide whether to:
  - land a RaBitQ-first SPIRE latency optimization,
  - make RaBitQ the explicit default for the validated SPIRE lane,
  - keep the current default unchanged with evidence and file narrower follow-up
    work, or
  - shelve with evidence if the remaining work is broader than a task-sized
    slice.
- Preserve the Task 73/75 100k high-recall recall@10 floor within `0.5 pp`.
- Preserve Task 76 10k behavior unless the RaBitQ evidence justifies a default
  change for the validated lane.

## Required Evidence

- Use `ecaz bench suite`; do not add ad hoc benchmark sweepers.
- Start from the Task 77 suite shape, switch the primary lane to RaBitQ, and
  capture before/after:
  - recall@10,
  - latency p50/p95/p99,
  - candidate funnel rows and scored-candidate counts,
  - retained/reranked candidate counts,
  - scoring-stage attribution,
  - TurboQuant comparison rows for the same points.
- For the first packet, explicitly report whether the latency problem is still
  dominated by candidate volume under RaBitQ. If not, rerank the P0 slices from
  the new evidence rather than carrying over the TurboQuant-only interpretation.
- Include Intel-local perf or stage-profile evidence for any claimed scan-side
  win.
- Run PG18 clippy:
  `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`.
- Run AWS only after a local slice clears the matched-recall p50 gate.

## Exit Criteria

- One RaBitQ-first SPIRE latency P0 slice either lands with `>=10%` p50
  improvement at matched 100k recall or is shelved with packet-local evidence.
- The closeout explicitly states whether the validated RaBitQ lane becomes the
  default, remains opt-in, or needs a separate default-policy task.
- No SPIRE recursion semantic change.
- Closeout packet records the decision and updates this task status.

## Closeout

Closed by `reviews/task-78/002-rabitq-suite-evidence/`.

The first RaBitQ-first P0 slice, bounded candidate cutoff from
`reviews/task-78/001-rabitq-candidate-cutoff/`, is shelved as a latency
optimization. Packet `002` measured parent RaBitQ baseline, current RaBitQ,
and current TurboQuant at the Task 77 100k high-recall points using
`ecaz bench suite`.

The current RaBitQ slice did not reduce the candidate surface or clear the
`>=10%` matched-recall p50 gate:

- nprobe64: `41.597 ms` baseline -> `41.757 ms` current at recall@10 `0.9825`.
- nprobe96: `60.881 ms` baseline -> `60.256 ms` current at recall@10 `0.9975`.
- nprobe128: `73.774 ms` baseline -> `74.951 ms` current at recall@10 `1.0000`.

The RaBitQ candidate funnel still scores `10,420,357`, `15,506,227`, and
`20,000,000` candidates across the 200 query rows while retaining `5,000`
and returning `2,000`. Scoring remains roughly `87-88%` of measured
candidate-path CPU time under RaBitQ, so the latency problem is still
candidate volume, not materialization or heap append.

RaBitQ remains the validated primary/default direction relative to TurboQuant:
current RaBitQ p50 is `41.757`, `60.256`, and `74.951 ms` at nprobe64/96/128,
versus TurboQuant `89.144`, `129.835`, and `167.193 ms` at identical recall.
This task does not flip a product default; a narrower default-policy task
should do that after a real candidate-selection win lands.

No SPIRE recursion semantic change was made.
