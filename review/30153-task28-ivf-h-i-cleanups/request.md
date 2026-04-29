# Review Request: Task 28 IVF H/I Cleanup Closure

## Scope

This packet closes the remaining H/I feedback from
`review/30152-task28-ivf-rabitq-score-hotpath/feedback.md`.

## H Items

H1-H4 were already closed by `91964193` and packet 30152.

H5 is packet-only and is now promoted in the local landing status packet: A3 is
closed for the local v1 claim, but flat rotating-window churn requires explicit
`posting_slack_percent`. Default `posting_slack_percent=0` preserves build size
and does not reserve churn headroom; packet 30142 measured about 24% growth on
the rotating-window workload without slack and flat behavior with slack enabled.

The consolidated A10 packet already points at packet 30152's corrected RaBitQ
rows. The A10 recommendation remains unchanged: keep `auto` unchanged in Task
28, recommend explicit `pq_fastscan, pq_group_size=8` for the larger
high-dimensional local IVF lane, and keep RaBitQ selectable but not a current
default candidate.

## I Items

### I1: RaBitQ encode/query-prep construction

Commit `78d2989d` adds a process cache for seeded RaBitQ quantizers keyed by
`(dimensions, seed, bits_per_dim)`. `IvfQuantizer::rabitq_quantizer()` now uses
that cache, so RaBitQ `encode_source` and `prepare_ip_query` reuse the same SRHT
state instead of reconstructing it per encode/prep call.

The existing construction-count regression now asserts encode + prepare cause
one construction total after clearing the test cache, and repeated score calls
do not add any construction.

### I2: test counter parallelism caveat

Commit `78d2989d` replaces the process-global construction counter atomics with
thread-local counters. Future tests can reset/read construction counts without
cross-thread clobbering from cargo's default parallel test runner.

## Validation

- `cargo test -p ecaz --lib am::ec_ivf::quantizer::tests::rabitq -- --nocapture`
- `cargo test -p ecaz --lib quant::rabitq::tests::qbit_code_len_scales_with_bits -- --nocapture`
- `git diff --check`

## Artifacts

- `artifacts/manifest.md`
