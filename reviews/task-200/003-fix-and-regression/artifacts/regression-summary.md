# Task 200 fix and regression decision

## Decision

No production read-path fix is required. The production A/B series is the
regression evidence: both stage-counter settings remain flat near 260–261 MB
for 300 one-backend queries, while the benchmark-only coverage helper is the
sole reproducer of multi-GB growth.

The code change is limited to diagnostic durability and fixture reuse:

- `--memory-series-output` streams RSS/HWM samples while a latency sweep runs.
- `--reuse-fixture` is opt-in, provenance checked, and records
  `fixture_decision action=reuse`; rebuild remains the default.
- Build-affecting mismatches (corpus, scale, graph degree, head cap, seed,
  codec, extension SHA/profile) reject reuse.

Validation completed:

- `cargo check -p ecaz-cli` passed, with the pre-existing unused-field warning.
- `ecaz bench suite audit` passed for the two-step Task 200 config.
- Counters-on reuse run completed; no PGDATA rebuild occurred.

The production physical read path was not changed, so the Task 200-specific
10k/50k/100k quantizer/index A/B closeout matrix is not applicable to this
diagnostic-only CLI change.
