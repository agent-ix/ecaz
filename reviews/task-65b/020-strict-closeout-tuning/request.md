# Packet 020: Strict Closeout Tuning

## Summary

This packet closes the two strict measurement gaps left by packet 019 and lands the tuned parallel build batch default.

Code change under review:

- `d5df80c88 Tune DiskANN parallel build batch default`
- Sets `ECDISKANN_DEFAULT_PARALLEL_BUILD_BATCH_SIZE` from `1` to `704`.
- The existing small-build cap keeps 10k builds at effective batch `64`, while 100k builds use effective batch `704`.

## Gate Results

### Synth10k

Task 65 accepted synth10k R32/L200 reference:

- L64/L200/L800 Recall@10: `0.1610 / 0.2625 / 0.3270`
- strict L200 floor: `0.2575`

Packet 020 run:

- config: `workers=8`, requested/effective batch `64`, R32/L240, `alpha=1.2`
- backend build total: `4232 ms`
- SQL build phase: `4.23 s`
- Recall@10 L64/L200/L800: `0.1610 / 0.2585 / 0.3295`

Verdict: synth L200 now passes the strict floor (`0.2585 >= 0.2575`).

### Real100k

Packet 001 real100k R32/L100 reference:

- L200 Recall@10: `0.9755`
- strict L200 floor: `0.9705`
- Task 65b time gate: `<= 30 s`

Packet 020 run:

- config: `workers=8`, requested/effective batch `704`, R32/L100, `alpha=1.2`
- backend build total: `28453 ms`
- SQL build phase: `28.47 s`
- Recall@10 L64/L128/L200: `0.9225 / 0.9645 / 0.9720`

Verdict: real100k passes both strict gates (`28.47s <= 30s`, `0.9720 >= 0.9705`).

## Validation

Packet-local validation artifacts:

- `artifacts/cargo-fmt-check.log`: passed; only existing stable-rustfmt warnings for unstable import grouping options.
- `artifacts/cargo-test-options-default.log`: passed, default reloption test.
- `artifacts/cargo-test-build-task65b.log`: passed, 6 Task65b build tests.
- `artifacts/install-after-default.log`: passed, installed the current PG18 extension after the default change.

Measurement artifacts:

- `synth10k-b64-l240-suite.json`
- `artifacts/synth10k-b64-l240-manifest.json`
- `artifacts/synth10k-b64-l240-results.jsonl`
- `real100k-b704-suite.json`
- `artifacts/real100k-b704-manifest.json`
- `artifacts/real100k-b704-results.jsonl`

## Review Ask

Please review this as the strict measurement/default closeout for Task 65b:

- synth10k strict recall floor is now met by the L240 closeout cell,
- real100k strict time + recall floor is now met by the b704 closeout cell,
- the tuned batch default has been landed in code.
