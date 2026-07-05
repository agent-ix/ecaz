# RaBitQ mutation triage

## Runs

Initial:

- Command: `cargo mutants --in-place --package ecaz-careful-hardening --file hardening/careful/src/../../../src/quant/rabitq.rs --output reviews/task-39/026-rabitq-mutation/artifacts/initial/rabitq.rs.mutants`
- Result: 456 mutants tested in 38m: 118 missed, 317 caught, 21 unviable.

Rerun after first test patch:

- Command: `cargo mutants --in-place --package ecaz-careful-hardening --file hardening/careful/src/../../../src/quant/rabitq.rs --output reviews/task-39/026-rabitq-mutation/artifacts/rerun/rabitq.rs.mutants`
- Result: 456 mutants tested in 44m: 27 missed, 408 caught, 21 unviable.

Final checkpoint run:

- Command: `cargo mutants --in-place --package ecaz-careful-hardening --file hardening/careful/src/../../../src/quant/rabitq.rs --output reviews/task-39/026-rabitq-mutation/artifacts/final/rabitq.rs.mutants`
- Result: 455 mutants tested in 74m: 9 missed, 423 caught, 21 unviable, 2 timeouts.

## Final missed mutants

| Mutant | Outcome | Verdict | Evidence |
| --- | --- | --- | --- |
| `rabitq.rs:409:32` replace `>` with `>=` in `encode_code` | missed | follow-up-test | Zero-norm branch boundary still needs a targeted zero-vector code-tail assertion. |
| `rabitq.rs:429:30` replace `>` with `>=` in `encode_code` | missed | follow-up-test | Zero denominator branch boundary still needs a targeted zero-vector `o_dot` assertion. |
| `rabitq.rs:585:50` replace `*` with `/` in `encode_code_centered` | missed | follow-up-test | Center-dot accumulator needs a stronger asymmetric-center assertion. |
| `rabitq.rs:588:41` replace `>` with `>=` in `encode_code_centered` | missed | follow-up-test | Zero-residual boundary needs a targeted centered-code tail assertion. |
| `rabitq.rs:728:49` replace `*` with `/` in `CenteredScorer::score_at` | missed | follow-up-test | Query-sign accumulator needs a stronger non-unit query vector assertion. |
| `rabitq.rs:750:24` replace `<` with `<=` in `CenteredScorer::score_at` | missed | follow-up-test | `o_dot` floor equality boundary needs direct coverage. |
| `rabitq.rs:752:35` replace `<` with `==` in `CenteredScorer::score_at` | missed | follow-up-test | Query-residual floor branch needs direct below/equal/above coverage. |
| `rabitq.rs:752:35` replace `<` with `<=` in `CenteredScorer::score_at` | missed | follow-up-test | Same query-residual floor boundary as above. |
| `rabitq.rs:993:38` replace `*` with `+` in `quantize_level` | missed | follow-up-test | Q-bit midpoint tests narrowed this class but one scale term still needs a non-roundtrip expected-value assertion. |

## Final timeout mutants

| Mutant | Outcome | Verdict | Evidence |
| --- | --- | --- | --- |
| `rabitq.rs:1127:30` replace `<` with `==` in `estimate_ip_impl` | timed-out | follow-up-test | Degenerate `candidate_o_dot` floor mutant timed out under cargo-mutants' auto 20s test timeout. |
| `rabitq.rs:1127:30` replace `<` with `<=` in `estimate_ip_impl` | timed-out | follow-up-test | Same floor-boundary class; should be covered with a smaller direct helper assertion or an increased mutation timeout. |

## Closed survivor classes

The added tests killed survivor classes for:

- quantizer and prepared-estimator accessors;
- wire-format version;
- q-bit pack/read roundtrips;
- q-bit midpoint quantize/dequantize behavior;
- persisted sidecar supported and unsupported lanes;
- binary sign lookup and sidecar word packing;
- centered raw-context access;
- degenerate centered scorer paths;
- direct estimator scalar and bound formula checks.
