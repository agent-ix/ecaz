# Task 47 / 007 — Wire cross-am-gate as PR-blocking CI

## Goal

Close one of Task 47's exit-criteria gaps: the `cross-am-gate`
Makefile target existed but was documented as "PR candidate,
report-first" and was not invoked from `.github/workflows/ci.yml`.
Promoting it into the `recall-cost-gates` job makes cross-AM
divergence PR-blocking alongside `recall-gate` and `cost-gate`.

## Scope of the change

- `.github/workflows/ci.yml`: added a `Cross-AM gate` step
  between the `Recall gate` and `Cost gate` steps in the
  `recall-cost-gates` job. Same invocation pattern, same
  PG18 socket env, same fixture load (PR fixtures are already
  generated and loaded earlier in the job).
- `docs/recall-floors.md`: dropped the "PR candidate,
  report-first" qualifier for `make cross-am-gate`; it is now
  unambiguously documented as PR cadence.

Source: 1 line in `.github/workflows/ci.yml`,
1 line in `docs/recall-floors.md`.

## Why this slice

Task 47 spec exit criteria #1 + #2 list `recall-gate` and
`cost-gate` as PR-CI requirements; the third gate
(`cross-am-gate`) is in the Makefile but was not in the CI job,
leaving cross-AM regressions unprotected at PR time. This packet
closes that gap with no behavioral change to the Makefile target.

## Code change

```diff
   - name: Recall gate
     run: make recall-gate ECAZ_ARGS="--database postgres --host /tmp/tqvector_pgrx_home --port 28818"

+  - name: Cross-AM gate
+    run: make cross-am-gate ECAZ_ARGS="--database postgres --host /tmp/tqvector_pgrx_home --port 28818"
+
   - name: Cost gate
     run: make cost-gate ECAZ_ARGS="--database postgres --host /tmp/tqvector_pgrx_home --port 28818"
```

## Validation

- The `recall-cost-gates` job already loads HNSW + IVF +
  DiskANN fixtures (the same prefixes
  `task47_gate_hnsw` / `task47_gate_ivf` / `task47_gate_diskann`
  the cross-am-gate config expects), so the new step adds no
  new setup requirements.
- `fixtures/gates/cross-am-gate-small.json` already exists in
  the repository with threshold rows for the three AMs.
- The change is CI-only — no code path under test changes; the
  next PR run will execute the new step.

## Reviewer direction

- Confirm the PR-blocking promotion is acceptable, or call out
  any reason cross-am-gate should stay nightly only.
- The 004 cross-am-consistency-metrics packet introduced the
  measurement; this packet closes the CI-wiring exit criterion.
