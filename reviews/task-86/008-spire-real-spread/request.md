# Review Request: SPIRE TurboQuant Real 10k/50k/100k Benchmark

## Scope

This packet supplies the production benchmark evidence missing from earlier Task 86 packets. It compares:

- Baseline: pre-LUT SPIRE TurboQuant source at `eda36f088dfafc1c3c379de7f3e0cfac888fae06`
- Current: SPIRE TurboQuant LUT source at `c200632f5835b3a0cd08938f3e9cdff5b836a8f9`

Both suites use `ecaz bench suite`, `ec_spire`, `storage_format=turboquant`, and the real10k/50k/100k DBPedia spread.

## Evidence

- Configs:
  - `reviews/task-86/008-spire-real-spread/suite-lutoff.json`
  - `reviews/task-86/008-spire-real-spread/suite-luton.json`
- Manifest:
  - `reviews/task-86/008-spire-real-spread/artifacts/manifest.md`
- Delta report:
  - `reviews/task-86/008-spire-real-spread/artifacts/benchmark-delta.md`
- Requirements audit:
  - `reviews/task-86/008-spire-real-spread/artifacts/requirements-audit.md`

## Main Result

Recall and storage are unchanged. SQL mean latency improves at every sweep point, ranging from about 2.4% to 4.5%. Pipeline query p50 improves by about 3.8% to 5.7%.

This validates the SPIRE no-QJL 4-bit TurboQuant dim-LUT scoring change as a real, no-format-change improvement. It does not validate TurboVec-style calibrated TQ+ yet; that remains a separate follow-up requiring its own real-corpus suite.

## Review Focus

- Whether packet 008 fully satisfies the benchmark gap raised on packets 005 through 007.
- Whether the requirements audit correctly separates measured improvements from remaining source/prototype hypotheses.
- Whether the benchmark should be promoted to a current local lane or rerun on another host class before promotion.
