# Task 230 packet 004 preregistration artifact manifest

- Head SHA: `5837f4bec64076769415645b00fb86b4c4e7294a`
- Task bucket: `reviews/task-230/004-full-scale-decision/`
- Packet: full-scale decision preregistration, seq-02
- Timestamp: 2026-08-29T05:54:54-07:00
- Lane / fixture / storage format / rerank mode: local Intel PG18;
  `ec_distann` row-heap control versus descriptor V4 / Graph V2 hot/cold;
  staged 1,536-dimensional real 10k/50k/100k corpora; no rerank variant
- Isolation: 20 fresh one-index-per-table fixtures, no reuse; two
  counterbalanced primary pairs per scale and matched fresh 100k secondary
  projection pairs; run directories live under `~/.ecaz/clusters`
- Results state: **none**. Only config audit and dry-run expansion exist.

## Seq-02 policy correction

- The suite config, its SHA, its 20-step expansion, and every accepted entry
  gate are unchanged from seq-01.
- Storage now has two correctly denominated gates: published candidate hot
  main-heap bytes at most 1.35× emitted logical raw-vector bytes, and total
  generation bytes at most 1.15× matched row-heap control bytes.
- Every preregistered §6 timing direction must be classified supported or
  falsified in the final decision even when the applicable numeric guardrail
  passes.
- No measurement result exists at this revision.

## `crates/ecaz-cli/suites/task230-hot-cold-10k-50k-100k.json`

- Commit: `5837f4bec64076769415645b00fb86b4c4e7294a` contains the suite config and
  no result artifact.
- Shape: 20 steps, 12 absolute recall thresholds, 12 primary steps across
  10k/50k/100k, and eight 100k secondary projection steps.
- Release guard: zero steps set `allow_debug_extension`.
- SHA-256: `e141ac65a7e18eaf4512509c549ba750e3106a2a045942e0eb6a5ac8fcc5437c`.

## `suite-audit.log`

- Command: `/home/peter/.cargo-target/debug/ecaz bench suite audit --config crates/ecaz-cli/suites/task230-hot-cold-10k-50k-100k.json --log-file reviews/task-230/004-full-scale-decision/artifacts/suite-audit.log`
- Result: exit 0, `audit passed: 20 steps`; required staged inputs exist.
- SHA-256: `a6a4ec8601b7bd5aff01c83e864314c642297b4256b8b7234a12b0bc457f64e9`.

## `suite-dry-run.log` and `suite-dry-run-manifest.json`

- Command: `/home/peter/.cargo-target/debug/ecaz bench suite run --config crates/ecaz-cli/suites/task230-hot-cold-10k-50k-100k.json --dry-run --manifest-output reviews/task-230/004-full-scale-decision/artifacts/suite-dry-run-manifest.json --results-output reviews/task-230/004-full-scale-decision/artifacts/suite-dry-run-results.jsonl`
- Result: exit 0; 20 dry-run steps, all expected command expansions present,
  no measurement results written.
- Manifest runner commit:
  `5837f4bec64076769415645b00fb86b4c4e7294a`.
- Config SHA-256 embedded in manifest:
  `e141ac65a7e18eaf4512509c549ba750e3106a2a045942e0eb6a5ac8fcc5437c`.
- SHA-256: log `3903797016eefecd756215b699659e1b6b6311780b3ce93916d830425537d92c`;
  manifest `fc1e21b1faeed7d38aa072090975615c006705ec6682eb0a430b6098f9c47495`.
