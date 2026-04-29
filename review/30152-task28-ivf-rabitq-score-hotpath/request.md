# Review Request: Task 28 IVF RaBitQ Score Hot Path

## Scope

This packet addresses reviewer finding H4 from
`review/30151-task28-ivf-local-landing-status/feedback.md`: the IVF RaBitQ
scan path rebuilt the seeded SRHT quantizer for every posting score.

## Code Change

Commit `91964193` makes `PreparedEstimator` carry the RaBitQ `bits_per_dim` and
score directly from prepared query state. `IvfQuantizer::score_ip_from_parts`
now calls `PreparedEstimator::estimate_ip(payload)` for RaBitQ instead of
constructing a `RaBitQQuantizer` per posting. `payload_len` also uses a static
RaBitQ code-length helper instead of constructing a quantizer.

The same commit handles the minor reviewer cleanups:

- adds the `aminsert`/VACUUM heap-TID uniqueness invariant comment in
  `insert.rs`
- documents the posting-slack separator page in `build.rs`
- exposes `relation_posting_slack_percent` in `ec_ivf_index_admin_snapshot`
  with PG18 test coverage

## Fixed RaBitQ Rows

Same existing A10 RaBitQ surfaces as packet 30144:
`nlists=64`, session `nprobe=48`, `rerank=heap_f32`, `rerank_width=750`.
Recall rows remain bounded to 20 queries and latency rows to 10 iterations.

| corpus | recall@10 | recall@100 | p50 | p95 | p99 | HWM |
|---|---:|---:|---:|---:|---:|---:|
| 10k | 1.0000 | 0.9930 | 344.2 ms | 401.3 ms | 413.1 ms | 68212 kB |
| 25k | 1.0000 | 0.9915 | 775.7 ms | 835.6 ms | 858.8 ms | 92996 kB |

For comparison, packet 30144 measured the broken RaBitQ path at 1947.8 ms p50
on 10k and 4973.0 ms p50 on 25k. The corrected path is still slower than the
selected PQ-FastScan row, but it is no longer a multi-second scan path caused
by quantizer reconstruction.

## Validation

- `cargo test -p ecaz --lib am::ec_ivf::quantizer::tests::rabitq -- --nocapture`
- `cargo test -p ecaz --lib quant::rabitq::tests::qbit_code_len_scales_with_bits -- --nocapture`
- `cargo pgrx test pg18 test_ec_ivf_admin_snapshot`
- `git diff --check`

## Measurement Commands

- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_qcmp10k_rabitq --profile ec_ivf --k 10 --iterations 10 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30152-task28-ivf-rabitq-score-hotpath/artifacts/latency_rabitq_10k_n64_w750_p48_i10_hwm.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_qcmp25k_rabitq --profile ec_ivf --k 10 --iterations 10 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30152-task28-ivf-rabitq-score-hotpath/artifacts/latency_rabitq_25k_n64_w750_p48_i10_hwm.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp10k_rabitq --profile ec_ivf --k 10 --queries-limit 20 --sweep 48 --rerank-width 750 --force-index --log-output review/30152-task28-ivf-rabitq-score-hotpath/artifacts/recall10_rabitq_10k_n64_w750_p48_q20.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp10k_rabitq --profile ec_ivf --k 100 --queries-limit 20 --sweep 48 --rerank-width 750 --force-index --log-output review/30152-task28-ivf-rabitq-score-hotpath/artifacts/recall100_rabitq_10k_n64_w750_p48_q20.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp25k_rabitq --profile ec_ivf --k 10 --queries-limit 20 --sweep 48 --rerank-width 750 --force-index --log-output review/30152-task28-ivf-rabitq-score-hotpath/artifacts/recall10_rabitq_25k_n64_w750_p48_q20.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp25k_rabitq --profile ec_ivf --k 100 --queries-limit 20 --sweep 48 --rerank-width 750 --force-index --log-output review/30152-task28-ivf-rabitq-score-hotpath/artifacts/recall100_rabitq_25k_n64_w750_p48_q20.log`

## Artifacts

- `artifacts/latency_rabitq_10k_n64_w750_p48_i10_hwm.log`
- `artifacts/latency_rabitq_25k_n64_w750_p48_i10_hwm.log`
- `artifacts/recall10_rabitq_10k_n64_w750_p48_q20.log`
- `artifacts/recall100_rabitq_10k_n64_w750_p48_q20.log`
- `artifacts/recall10_rabitq_25k_n64_w750_p48_q20.log`
- `artifacts/recall100_rabitq_25k_n64_w750_p48_q20.log`
- `artifacts/manifest.md`
