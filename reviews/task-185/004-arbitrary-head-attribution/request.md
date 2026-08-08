---
task: 185
packet: 004-arbitrary-head-attribution
role: coder
status: open
date: 2026-08-07
seq: 01
head: 815ff3551
---

# Task 185 arbitrary-head attribution checkpoint

This packet records the narrow implementation needed to move beyond the
returned-seed basin diagnostic in packet 003. It does not claim a selector,
an A/B result, or a production change.

## Change

Commit `815ff3551` adds a benchmark-only SQL endpoint,
`ec_distann_physical_head_candidate_trace_benchmark(regclass, real[], integer,
integer)`. It exact-scores the persisted head membership, selects one ranked
member by 1-based position, and reruns the physical scan with only that
candidate. On sharded fixtures it uses the existing exact sharded head path.

The endpoint is feature-gated by `distann-head-attribution-benchmark`, does not
alter the production seed selector, and does not persist a policy. Candidate
positions outside `1..=4096` are rejected, and a position beyond the actual
head size fails explicitly.

## Validation

- PG18 feature build with `distann-head-attribution-benchmark`: pass.
- PG18 featureless build: pass.
- `git diff --check`: pass.
- Repository-wide stable `cargo fmt --all -- --check` remains non-clean because
  the checkout contains existing nightly-only import-format configuration and
  unrelated formatting differences; no formatter rewrite was applied.

No benchmark was run from this packet. The next checkpoint must wire this
surface into `ecaz bench suite` and run a bounded arbitrary-head diagnostic on
the fixed 100k control before any selector implementation or A/B claim.

## Review focus

Please review that the endpoint is genuinely benchmark-only, uses the exact
persisted head rather than the normal returned-seed list, preserves the
physical scan and trace semantics, and does not change featureless production
behavior.
