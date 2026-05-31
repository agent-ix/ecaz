# Task 78: SPIRE TurboQuant Scoring-Kernel Optimization

Status: proposed (2026-05-31, split from Task 77 no-slice closeout)
Owner: coder (to be assigned). One coder, one branch.
Priority: 1 (required before revisiting SPIRE high-recall defaults)

## Why

Task 77's Intel-local candidate attribution packet showed that high-recall
SPIRE scan latency is dominated by approximate quantized scoring, not by row
materialization or heap maintenance:

- `tg64/nprobe64`: `10,420,357` candidates over 200 queries; scoring is about
  `82.9%` of measured candidate-path time.
- `tg96/nprobe96`: `15,506,227` candidates over 200 queries; scoring is about
  `82.1%` of measured candidate-path time.
- `tg128/nprobe128`: `20,000,000` candidates over 200 queries; scoring is about
  `83.2%` of measured candidate-path time.

The row-materialization and heap-retention slices Task 77 was created to test
are below the `10%` p50 improvement floor. Object reads are also measurable
(`17.934 ms` p50 at tg96/nprobe96), but reducing that cost belongs with
storage-format/object-layout work rather than a SPIRE-local materialization
slice. The next useful work is therefore a scoring-kernel or storage-format
change that makes each candidate cheaper without changing selected leaves,
recursion semantics, recall floors, or defaults.

## Scope

- Profile the SPIRE `storage_format = 'turboquant'` candidate scorer at the
  same 100k high-recall points Task 77 measured.
- Account for object-read cost at those points and decide whether any
  storage-format/object-layout slice is task-sized, or whether it belongs in a
  broader format redesign.
- Compare exact TurboQuant scoring against existing approximate/no-QJL/LUT
  candidate kernels where available.
- Decide whether to:
  - land a TurboQuant score-kernel optimization,
  - add a SPIRE-local scoring mode guarded by recall evidence,
  - move high-recall SPIRE to an existing RaBitQ/PqFastScan-style storage
    format task, or
  - shelve with evidence if the remaining work is broader than a task-sized
    slice.
- Preserve the Task 73/75 100k high-recall recall@10 floor within `0.5 pp`.
- Preserve Task 76 10k behavior and current defaults unless a separate default
  task is reopened with broader evidence.

## Required Evidence

- Use `ecaz bench suite`; do not add ad hoc benchmark sweepers.
- Start from the Task 77 suite shape and capture before/after:
  - recall@10,
  - latency p50/p95/p99,
  - candidate funnel rows,
  - scoring-stage attribution,
  - fixed-candidate score-kernel microbench or replay evidence.
- Include Intel-local perf or stage-profile evidence for any claimed scan-side
  win.
- Run PG18 clippy:
  `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`.
- Run AWS only after a local slice clears the matched-recall p50 gate.

## Exit Criteria

- One scoring/storage-format P0 slice either lands with `>=10%` p50 improvement
  at matched 100k recall or is shelved with packet-local evidence.
- No SPIRE recursion semantic change.
- No default change unless a follow-up default-policy task is explicitly
  reopened.
- Closeout packet records the decision and updates this task status.
