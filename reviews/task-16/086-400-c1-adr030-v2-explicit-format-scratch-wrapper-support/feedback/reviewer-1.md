## Feedback: Explicit Format Scratch Wrapper Support

Read `scripts/prepare_real_corpus_scratch.sh` (added
`--storage-format`), `scripts/bench_sql_latency_verified.sh` (added
resolution precedence), and
`scripts/tests/test_bench_sql_latency_verified.py`.

### What's right

- **Explicit precedence for `--index-name`.** The resolution order
  is `--index-name` > `<prefix>_<storage_format>_m{N}_idx` > legacy
  `<prefix>_m{N}_idx`. That's exactly the right ordering — operator
  override wins, derivation is the ergonomic default, legacy shape
  is still supported. Documented clearly in the packet body.
- **Verified launcher forwards the derived `--index-name` down to
  the delegate `bench_sql_latency.sh`.** Without this, the planner-
  verification step and the measured cell could resolve different
  indexes. That silent-divergence class of bug is exactly what
  forwarding the derived name prevents.
- **Python regression test exercises the actual launcher path.**
  `test_verified_launcher_derives_explicit_storage_format_index_name`
  asserts banner + derivation + run composition through the fake
  psql harness. That proves the three-step integration works in
  sequence, not just the derivation in isolation.
- **Scratch prepare wrapper forwards `--storage-format` through to
  the loader.** Closes the last operator-visible gap from 398 —
  users don't have to hop between two script layers that disagree
  on naming.

### Concerns

1. **Invalid `--storage-format` handling path untested.** The
   packet claims "validates the value and forwards it"; the
   regression covers the successful path. A single invalid-value
   assertion would confirm the validation actually rejects
   (rather than silently forwards) an unknown format.
2. **Docs updated, no runbook example.** Same concern as 398 —
   a worked example in `docs/RECALL_REAL_CORPUS.md` showing the
   exact sequence (prepare → load → bench latency, across both
   formats) would be the landing artifact operators will actually
   use.
3. **Linker gap is irrelevant here.** This packet is scripts +
   Python only; `scripts/tests/run.sh` is the load-bearing
   validation and it passed. The packet is well-proven within its
   own scope.

### Observation

Right-sized packet to end the arc on. The remaining meaningful
work, as the packet correctly notes, is no longer harness plumbing
— it's running the recall/latency lanes against the explicit
families and comparing against the task-15 landing bar. That's the
execution step, not another implementation slice.

### Meta-observation on the 378–400 arc

23 packets of narrow slices, mostly well-separated. Two systemic
gaps run through all of them:

1. **`cargo test` and `cargo pgrx test pg17` have not run locally
   for the entire arc.** The workstation is linker-blocked on
   PostgreSQL symbols. That means every pg test introduced in the
   arc (round-trip, insert assertions, vacuum assertions, bootstrap,
   small-dim build, canonical env, coexistence smoke) is proven
   only by `cargo check --tests` + clippy + manual inspection.
   Before merge, whichever CI lane does run pgrx tests needs to be
   confirmed green explicitly — ideally with a packet that names
   each new test and confirms it passed.

2. **Task 15's definition of done includes "Insert + vacuum
   round-trip on both formats" and "passes the 50k real seam
   recall harness."** 393 closes the round-trip in pg tests; 398
   + 399 + 400 stand up the real-corpus harness plumbing. The
   actual 50k run has not happened (correctly framed as next-slice
   execution work). Merge-readiness pivots on that run producing
   acceptable numbers.
