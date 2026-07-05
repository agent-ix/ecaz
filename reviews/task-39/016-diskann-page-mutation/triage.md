# Task 39 DiskANN Page Mutation Triage

Code checkpoint: `33e6f6f86d1c6d302db46c53bef83ce6c97050f4`

Initial run:

- Command: `make mutants MUTANTS_MODULE=src/am/ec_diskann/page.rs MUTANTS_OUTPUT_DIR=reviews/task-39/016-diskann-page-mutation/artifacts MUTANTS_JOBS=2`
- Result: 11 mutants tested, 2 missed, 7 caught, 2 unviable.
- Raw survivor list: `artifacts/page.rs.mutants/mutants.out/missed.txt`.

Fix:

- Added exact wire-bit assertions for DiskANN metadata payload flags.
- Replaced `1 << 0` with the literal `0b0000_0001` for `PAYLOAD_FLAG_BINARY_SIDECAR` because `1 << 0` and the mutant `1 >> 0` are equivalent.

Final run:

- Command: `make mutants MUTANTS_MODULE=src/am/ec_diskann/page.rs MUTANTS_OUTPUT_DIR=reviews/task-39/016-diskann-page-mutation/artifacts/final MUTANTS_JOBS=2`
- Result: 10 mutants tested, 0 missed, 8 caught, 2 unviable.
- Raw outcome: `artifacts/final/page.rs.mutants/mutants.out/outcomes.json`.

| Initial missed mutant | Verdict | Evidence |
| --- | --- | --- |
| `page.rs:26:47 replace << with >>` | removed equivalent | `1 << 0` and `1 >> 0` both produce `1`; code now uses literal `0b0000_0001`, and `la_005b_empty_clears_cold_rerank_flag` asserts the value. |
| `page.rs:28:52 replace << with >>` | killed | `la_005b_empty_clears_cold_rerank_flag` now asserts `PAYLOAD_FLAG_COLD_RERANK_PAYLOAD == 0b0000_0100`. |

Residual:

- Final missed mutants: none.
- Final unviable mutants: 2, recorded in `artifacts/final/page.rs.mutants/mutants.out/unviable.txt`.
