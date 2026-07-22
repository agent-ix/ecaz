# Task 194 attribution manifest

- Implementation head: pushed counter commits on `task-192-194-owner-transport`.
- Validation: offline PG18 library and `ecaz-cli` checks passed.
- Physical run: topology, serving, recall, and latency completed; stale CLI
  rejected the new 28-row schema after emitting the measurements.
- Attribution summary: `artifacts/attribution-100k.md`.
- Raw latency SHA256: `f00739db1ca6498319b2795a4c2fec4d10ff128a334dccd5fdc82bd80ab28f64`.
- Raw recall SHA256: `e4aa8ad4fd0328b37970c2e3c5edca0bc7657e4670736b68f6640f00b383fd4d`.
- Canonical suite config: `artifacts/suite/task194-suite.json`.
- Suite audit and dry-run passed with the updated 28-stage/19-work-row CLI;
  outputs are `artifacts/suite/dry-run.log` and
  `artifacts/suite/dry-run-manifest.json`.
- The completed measurement was retained from the direct fixture invocation;
  the suite dry-run reproduces the exact invocation and prevents the stale
  parser failure from being mistaken for a measurement failure.
- Decision input: Task 187's accepted baseline remains the only numerical
  traversal evidence.
