## Feedback: External Recall Smoke Storage Formats

Read `create_external_recall_smoke_tables`,
`create_external_recall_smoke_indexes`,
`external_recall_index_prefix/name`,
`assert_external_recall_smoke_probe`, and
`test_tqhnsw_recall_external_smoke_500_formats` in `src/lib.rs`.

### What's right

- **In-tree smoke surface matches the operator harness from 398.**
  After this, the Rust external recall smoke proves exactly the
  contract the loader/runner scripts implement: shared tables,
  three index families (legacy/default, explicit turboquant,
  explicit pq_fastscan) on one staged corpus.
- **Split of fixture helpers into tables + indexes is the right
  factoring.** Mirrors the shape of the loader changes in 398. A
  test that mixed table creation with per-format index creation
  couldn't have proven coexistence cleanly.
- **Shared assertion helper (`assert_external_recall_smoke_probe`)
  runs identical checks against each family.** Bug-hunting
  ergonomics: if only one family fails, the divergence is
  localized to the format-specific code path, not to a
  test-harness difference.
- **Determinism assertion across reruns is present.** Summary
  determinism + gate-row-per-checkpoint both covered. That catches
  a class of bugs (nondeterministic scoring, state leakage across
  queries) that the earlier-arc determinism packet 361 worked to
  eliminate.

### Concerns

1. **Test is `#[ignore]`.** The packet description flags that it
   didn't run on this workstation. Whatever CI lane runs ignored
   tests needs to actually run this one — otherwise the "in-tree
   proof of coexistence" is aspirational. This is the most
   important deferred test in the arc.
2. **Three families, one corpus, one set of assertions — no
   comparison between families.** The test proves each family
   passes, not that the families return equivalent results. A
   delta assertion (e.g., "legacy and explicit-turboquant
   agree on ranking within tolerance") would be a stronger
   coexistence check and directly relevant to task 15's ordering
   of "land both as peers." Probably followup.
3. **Assertions are shape-focused (row counts, metric ranges,
   determinism), not recall-focused.** That matches "smoke" as
   framing, but a landing claim of "PqFastScan is first-class"
   needs to pair this with actual recall/latency deltas from the
   real-corpus lane. Noted as next-slice in packet body.

### Observation

This is the in-tree companion to 398. Together they make the task-15
landing story auditable from code alone — "which proof does this
claim rest on?" maps to one named test. Critical that the ignored
test actually runs somewhere before merge.
