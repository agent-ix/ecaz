# Task 215 release-contract manifest

- Task bucket: `reviews/task-215/001-release-contract/`
- Contract checkpoint: `7055a8ec5`
- Scope: BW64/H8 productionization contract only; no benchmark run
- Control: `beam_width=4`, `hop_rounds=100`, `candidate_heap_limit=32`,
  production-derived head seed count 32
- Candidate: `beam_width=64`, `hop_rounds=8`,
  `candidate_heap_limit=32`, production-derived head seed count 128
- Binary requirement: normal PG18 release build, without
  `distann-head-attribution-benchmark`
- Compatibility surface: session GUC defaults only; index bytes, generation
  format, placement, and persisted head identity are unchanged

The release A/B and all measured artifacts belong to packet 003. This packet
contains no benchmark output and makes no promotion claim.
