# Task 71 Review Request: Single-TID Build Posting Staging

## Scope

Code commit under review:

- `59aa48890 Avoid build-time posting TID allocation`

This slice adds a build-time posting insert path for the common IVF build case:
one posting tuple with exactly one heap TID. It writes the same on-disk bytes as
the generic `IvfPostingTuple` encoder, but avoids allocating a
`Vec<ItemPointer>` for every build row.

The deterministic list order, tuple order, payload encoding, page layout, and
directory construction are unchanged.

This is a measured staging improvement, not Task 71 completion. The current
worker curve is still Amdahl-capped by leader training plus remaining posting
staging work.

## Validation

Packet-local artifacts are under
`reviews/task-71/006-single-tid-posting/artifacts/`.

- `cargo test single_heaptid_posting_encoder_matches_generic_encoder`
  - passed.
  - Proves the new single-TID encoder matches the generic posting encoder
    byte-for-byte for the build-time one-TID shape.
- `cargo check --no-default-features --features pg18`
  - passed.
- `./target/debug/ecaz --log-file reviews/task-71/006-single-tid-posting/artifacts/install-after-single-tid-posting.log dev install ecaz-pg-test --pg 18`
  - passed; backend artifact assertion passed.
  - Installed dylib SHA:
    `2b8c5b56a3f1cf0dbee74401e6009f0a5a340c7887f130ed1b209ad92a749ade`.
- `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-71/006-single-tid-posting/artifacts/probe-single-tid-posting-w8.log dev test ivf-parallel-build-probe --drop-first --workers 8 --prefix task71_probe_w8_single_tid_posting`
  - passed without approval escalation.
  - Loader artifact:
    `artifacts/probe-load-real10k-w8-single-tid-posting.log`.
  - Key build line:
    `built task71_probe_w8_single_tid_posting_idx in 417.46ms`.
  - Key timing row:
    `requested_workers=8 workers_launched=7 heap_ingest_us=35622 train_model_us=263644 stage_build_plan_us=108117 stage_pq_train_us=15811 stage_centroids_us=186 stage_assign_us=29179 stage_postings_us=62898 stage_directory_us=4 flush_build_plan_us=2278`.

## Comparison

The immediately previous packet 005 w8 real10k probe recorded:

- build time `463.36ms`
- `stage_build_plan_us=144585`
- `stage_postings_us=92930`

This slice recorded:

- build time `417.46ms`
- `stage_build_plan_us=108117`
- `stage_postings_us=62898`

So the local one-cell probe shows the intended posting-staging reduction:
`stage_postings_us` improved by about `30.0ms` (~32%). Full build time improved
by about `45.9ms` on this run, but the remaining `train_model_us=263644` keeps
the full build far from the multi-x Task 71 exit criterion.

## Review Focus

- Whether the single-TID posting encoder preserves the generic encoder bytes
  and remains scoped to build-time one-TID postings.
- Whether the next Task 71 slice should target leader training or a larger
  deterministic posting-staging restructure, since this allocation reduction is
  not enough by itself.
