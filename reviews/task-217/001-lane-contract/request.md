# Task 217 — same-generation lane contract

This packet pre-registers the measurement contract that unblocks owner-side
materialization work. The lane uses the existing `benchmark_seed_variants`
surface, which builds one physical generation once and evaluates multiple
read-time runtime switches against it. It does not change the production read
path or any shipped default.

The authoritative implementation is commit `15834e2e4`. Each physical arm
now reads `ec_distann_active_epoch.epoch_fingerprint`, emits a
`physical_benchmark_generation` row, and fails closed if a later arm observes
a different identity. The optional `same_generation_recall_pair` contract
compares the two prediction files byte-for-byte and fails the run on any
difference.

The deliberate A/B in the implementation packet is a read-time scoring switch
(`rabitq` versus `exact_neighbor`); it is intentionally not presented as a
construction-affecting change. The A/A pair is the trust gate: identical
runtime settings must produce byte-identical predictions on the same epoch.

NFR-021/NFR-022 are pre-registered in the implementation SuiteConfig for all
four arms: A/A control and candidate, plus the A/B control and candidate.
Decision-bearing controls are conforming and no traversal replica is enabled.

## Review focus

- The epoch fingerprint is the correct immutable generation identity.
- Equality is checked per physical arm before its recall/latency work.
- A/A byte identity is asserted from prediction artifacts, not inferred from
  recall aggregates.
- The SuiteConfig is the only runner for the proof; no shell sweep is used.

See `reviews/task-217/002-lane-implementation/` for the implementation,
config, validation, and 100k proof artifacts.
