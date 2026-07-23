# Task 197: DistANN Multinode Release-Profile Preflight

Status: **complete — outside-reviewed ACCEPT; PROMOTE** (2026-07-23).
Priority: P1 benchmark-integrity follow-up.

## Why

Task 194 lost several hours because `cargo pgrx test` or a non-release install
replaced the installed PG18 `ecaz.so` with a debug binary. The physical 100k
build then remained CPU-bound for hours instead of completing in roughly 15
minutes. The generic `ecaz bench suite` release guard currently selects direct
`latency` and `recall` steps only. A `distann-local-multinode` step launches its
own PostgreSQL instances and reports extension provenance only after the long
physical build, so the generic guard does not protect this lane.

## Goal

Fail fast before corpus loading or physical generation construction when a
benchmarking `distann-local-multinode` fixture starts from a non-release
extension.

1. Split extension/bootstrap setup from corpus setup if necessary. Immediately
   after the fixture nodes load the extension, query `ecaz_build_profile()` and
   `ecaz_build_git_sha()` on every node, before corpus load or generation build.
2. Require a unanimous `release` profile and git SHA before expensive setup;
   report the node/port and observed provenance on mismatch.
3. Provide an explicit debug override only for intentional diagnostic runs;
   thread it through `ecaz bench suite` rather than using an environment or
   shell workaround.
4. Emit and flush the successful preflight to the packet-local fixture log
   before the build begins; successful suites also parse it into the structured
   result stream.
5. Reuse the same fixture-level helper for other self-hosted DistANN benchmark
   steps rather than duplicating profile logic.

## Evidence

- Focused CLI tests prove release acceptance, debug rejection, mixed-node
  rejection, and explicit-override behavior.
- A dry-run/audit proves the option is represented in the structured suite
  command.
- A short local fixture smoke proves the preflight is emitted before physical
  construction starts. This tooling-only change does not alter quantizer,
  index, scan, rerank, posting, or storage behavior and therefore does not
  require a 10k/50k/100k performance matrix.

## Implementation outcome

The fixture now creates the extension and validates every node before entering
either the physical or replicated-serving setup path. The validator requires a
non-empty unanimous git SHA and build profile, rejects every non-release
profile by default with node/port diagnostics, and accepts a non-release
profile only through the suite-serialized `allow_debug_extension` field. The
accepted attestation is reused by the later physical benchmark provenance row.

Packet 001 contains the focused release/debug/mixed/override tests, suite
expansion and structured-result tests, audit/dry-run manifests, and a successful
physical fixture smoke. Its log records the flushed release preflight before
`physical_setup_start`, and `results.jsonl` contains the corresponding
`multinode_release_preflight` row. Outside review accepted the implementation
and evidence with no findings and marked the task merge-ready.

## Review packet

`reviews/task-197/001-multinode-release-preflight/`.

## References

- Task 194 packet 005 reviewer feedback (debug installed-library root cause).
- Task 194 packet 008 corrected release run.
- SPIRE Tasks 141--146 benchmark-integrity precedent.
