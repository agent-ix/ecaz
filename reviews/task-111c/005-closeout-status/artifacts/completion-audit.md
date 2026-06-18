# Task 111c Completion Audit

Task: `plan/tasks/111c-ivf-page-aware-scatter-scorer.md`

Conclusion: Task 111c is complete as a **stopped/no-promote** score-in-place
experiment. The reference TQ path is implemented, correct, and opt-in, but the
benchmark gate failed after the final requested locality lever. The broad
codec/ISA fanout and default promotion are intentionally not pursued.

## Acceptance Criteria

### AC1: Page-aware scatter scorer implemented for active codecs across AVX2, SVE2-128, and NEON, behind the gate, with per-ISA coverage gates met.

Status: stopped after reference gate failure.

Evidence:

- `reviews/task-111c/001-*` through `004-*` implement and measure the TQ
  reference path.
- Packet 002 reviewer feedback set the hard gate: page scatter must beat the
  dense/copy baseline before fanout.
- Packet 004 implements the final requested reference-path lever and still
  measures scatter slower than copy fallback.

Decision:

- Do not fan out a measured-losing access pattern across RaBitQ/grouped-PQ and
  AVX2/SVE2/NEON.
- Keep the implemented TQ path behind `ec_ivf.columnar_page_scatter`.
- Commit `c3ae49cd586d0451083aa743ed55d145a78465d9` defaults that GUC to `off`.

### AC2: Scores are bit-identical to the 111b copy-based scan.

Status: complete for the implemented TQ reference path.

Evidence:

- `reviews/task-111c/002-page-scatter-explain-ab/` added the multi-page
  `test_ec_ivf_columnar_page_scatter_matches_copy_scan` fixture.
- `reviews/task-111c/004-page-run-payload-refs/artifacts/cargo-pgrx-test-pg18-page-scatter-equivalence.log`
  reran the equivalence test after the page-run payload-ref lever:
  `1 passed; 0 failed; 2130 filtered out`.

### AC3: Per-group/per-scan assembly copy-bytes counter drops to ~0 on the columnar path.

Status: complete for the implemented TQ reference path.

Evidence:

- Packet 002 added `Columnar Payload Bytes Borrowed` observability.
- Packet 004 warmed A/B:
  - scatter `Columnar Logical Bytes Copied`: 0;
  - scatter `Columnar Payload Bytes Borrowed`: 18,358,272;
  - copy fallback `Columnar Logical Bytes Copied`: 18,887,163.

### AC4: SIMD flush widths reach configured W for every spanning quant mode.

Status: complete only for the implemented TQ reference path; broad spanning-mode
fanout stopped by the gate.

Evidence:

- Packet 004 preserved cross-page accumulation while deriving refs by page run.
- Packet 004 warmed A/B kept `Dense Coalesced Flushes`: 109 for both scatter and
  copy fallback, showing the page-run lever did not regress logical batch width.

Decision:

- No RaBitQ/grouped-PQ fanout because AC6 failed on the reference path.

### AC5: Recall and NDCG unchanged across the matrix.

Status: not expanded to a full matrix because the reference latency gate failed.

Evidence:

- The implemented TQ reference path has bit-identical score/output coverage
  against the copy fallback.
- The 111b benchmark matrix remains the recall/NDCG baseline for the columnar
  copy path.

Decision:

- A full TQ + RaBitQ matrix is not justified for scatter after AC6 failed.

### AC6: Benchmark packet shows score-in-place beats Approach A/copy at high-recall cells and makes a promote/iterate decision.

Status: failed; closeout decision is stop/no-promote.

Evidence:

- Packet 002 first measured scatter slower than copy fallback.
- Packet 003 reduced heap-TID overhead but still lost.
- Packet 004 implemented the requested page-run payload-ref locality lever and
  still measured scatter slower than copy fallback on the warmed 50k TQ
  reference cell:

| Cell | Approx scan us | Exec ms |
| --- | ---: | ---: |
| Page scatter, page-run refs | 30,141 | 34.536 |
| Copy fallback same head | 18,986 | 23.199 |

Decision:

- Do not promote page scatter.
- Stop 111c codec/ISA fanout.
- Default `ec_ivf.columnar_page_scatter` to `off`.
- Mark 111d won't-pursue for this line because pre-transpose does not fix the
  scattered-read locality gap.
- Carry forward the lesson for any future layout work: a new layout must beat
  contiguous copy plus sequential scoring directly; zero-copy alone is not
  sufficient evidence.

## Closeout Summary

Task 111c answered the key design question with committed evidence. The answer
is negative for this page-scatter design: removing the assembly copy does not
beat the locality of copy-then-sequential-score. The implemented path remains
valuable for diagnostics and equivalence checks, but the production/default path
stays on the faster Task 111b copy fallback.
The immediately dependent 111d pre-transpose task is closed as won't-pursue for
this line; reopening it requires a fresh design and a direct copy-fallback gate.
