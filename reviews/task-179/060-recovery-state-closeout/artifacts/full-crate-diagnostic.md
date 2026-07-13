# Full-crate pgrx diagnostic (not the Task 179 gate)

An exploratory `RUST_TEST_THREADS=1 cargo pgrx test pg18` run was stopped after
three pre-existing non-DistANN unit failures made a full-crate green result
impossible on this host. Each was reproduced independently from the generated
test binary with `--exact --nocapture --test-threads=1`:

1. `am::common::candidate_batch::tests::turboquant_lut_batch_matches_scalar_tail`
   compares bit-identical scalar/batch results and differs by two `f32` bit
   patterns (`1022068201` vs `1022068199`) on the native x86_64 path.
2. `am::ec_diskann::quantizer::tests::diskann_turboquant_prepared_prefilter_batch_scores_and_records_counters`
   differs by six bit patterns (`3151609872` vs `3151609866`).
3. `quant::prod::tests::tiled_lut_query_prep_rejects_qjl_active_lane` reaches an
   older `left: 8 / right: 16` assertion before its expected QJL panic string.

These tests predate Task 179, do not exercise ec_distann, and are unchanged by
the Task 179 branch. Expanding this closeout into TurboQuant production/test
semantics would be unrelated scope. The decision-grade pgrx gate is therefore
the complete `distann`-named test surface recorded in `distann-pg18-green.log`.
