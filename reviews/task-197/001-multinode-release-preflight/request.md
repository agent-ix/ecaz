# Task 197 review request: multinode release-profile preflight

Please review implementation commit `be2f3497f0bbae16a861bcbb91ac44bab7595e92`
and this packet for Task 197 closeout.

## Scope

- The self-hosted DistANN fixture now creates `ecaz` on each node and queries
  `ecaz_build_git_sha()` plus `ecaz_build_profile()` before any corpus or
  physical-generation setup.
- A shared validator requires non-empty, unanimous SHA/profile provenance and
  a release profile. Failures identify the observed node and port.
- `allow_debug_extension` is an explicit `distann-local-multinode` suite field
  that expands to `--allow-debug-extension`; it permits only a unanimous
  diagnostic profile and never suppresses mixed-node rejection.
- The successful preflight is emitted through the flushing output surface and
  parsed into a `multinode_release_preflight` result row.
- The same early helper protects physical and replicated-serving fixture modes;
  the physical benchmark reuses its attestation rather than querying a second,
  late provenance loop after construction.

## Evidence

See `artifacts/manifest.md`. In summary:

- four focused validator tests pass;
- suite expansion and structured-result parser tests pass;
- suite audit and dry-run pass;
- the suite-driven physical smoke succeeds, with the release preflight at log
  line 4, `physical_setup_start` at line 5, and a structured passing result;
- a supplemental two-node replicated-control smoke uses the same helper,
  reports unanimous release/SHA provenance before either node loads rows, and
  ends with its direct distinct-recall gate passing;
- no corpus TSV, PostgreSQL node log, truth cache, or polling exhaust is
  committed.

This is tooling-only and changes no quantizer, index, scan, rerank, posting, or
storage behavior. Per the task definition, a 10k/50k/100k performance matrix is
not required.

## Requested decision

Please return ACCEPT/PROMOTE or actionable findings. In particular, verify the
pre-setup ordering, node/port mismatch diagnostics, override scoping, cleanup
on preflight failure, suite command representation, and structured evidence.
