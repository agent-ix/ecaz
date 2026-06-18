# Review Request: Task 111c Closeout Status

## Scope

This packet requests review of the Task 111c closeout/status update and the
default-off runtime gate for the page-scatter path.

Code checkpoint:

- `c3ae49cd586d0451083aa743ed55d145a78465d9`
  (`Task 111c: default page scatter off`)

Changed files:

- `src/am/ec_ivf/options.rs`
- `plan/tasks/111c-ivf-page-aware-scatter-scorer.md`
- `plan/tasks/111b-ivf-columnar-frozen-list-format.md`
- `plan/tasks/111d-ivf-pretransposed-canonical-block-geometry.md`
- `plan/tasks/README.md`
- `reviews/task-111c/005-closeout-status/artifacts/manifest.md`
- `reviews/task-111c/005-closeout-status/artifacts/completion-audit.md`

## Why Close Now

Task 111c's reference path and decision gate have been exercised:

- packet 001 proved genuine borrowed payload zero-copy for the TQ reference
  path;
- packet 002 added bit-exact equivalence and EXPLAIN A/B evidence, but showed
  scatter was much slower than the copy fallback;
- packet 003 removed per-posting heap-TID allocation and improved scatter, but
  still did not close the gap;
- packet 004 implemented the extra reviewer-requested locality lever
  (page-run payload refs accumulated across pages), improved scatter slightly,
  and still lost to the copy fallback.

The warmed packet 004 A/B result is the deciding gate:

| Cell | Approx scan us | Exec ms |
| --- | ---: | ---: |
| Page scatter, page-run refs | 30,141 | 34.536 |
| Copy fallback same head | 18,986 | 23.199 |

That fails the "beat dense/copy before fanout" gate from packet 002 feedback.
The closeout decision is therefore: **do not promote page scatter, stop codec/ISA
fanout, and keep scatter as an opt-in diagnostic path.**

## Runtime Gate

`ec_ivf.columnar_page_scatter` now defaults to `off` outside tests. The path can
still be enabled explicitly for diagnostics and equivalence work, but ordinary
columnar scans use the Task 111b logical-copy fallback because it is the faster
measured path.

## Validation

Artifacts are under `reviews/task-111c/005-closeout-status/artifacts/`.

- `cargo-build-pg18.log`
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 1m 13s`

No new benchmark was run for this closeout packet. The benchmark decision is
based on the already committed packet 004 `ecaz bench suite` A/B artifacts.

## Review Focus

- Does the closeout accurately record that 111c is complete as a stopped/no-
  promote experiment rather than a successful promotion?
- Is defaulting `ec_ivf.columnar_page_scatter` off the right runtime gate after
  the packet 004 result?
- Does `artifacts/completion-audit.md` preserve the evidence trail and make the
  fanout stop condition explicit enough for 111d/future work?
