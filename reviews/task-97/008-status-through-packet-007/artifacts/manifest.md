# Task 97 Packet 008 Artifact Manifest

- head SHA: `ee59192220c956d6eeebade6a73715d3e5035f4b`
- task bucket: `reviews/task-97/008-status-through-packet-007/`
- lane: coder-1 LUT lane, Task 97 TurboQuant QJL block kernel family
- host: local x86_64 Linux
- scope: metadata/status refresh and local readiness matrix
- files changed:
  - `plan/tasks/97-tq-qjl-block-kernel-family.md`
  - `plan/tasks/README.md`
- CI/AWS: not run

## Evidence Ledger

| Requirement | Current evidence | Status |
| --- | --- | --- |
| Phase 0 QJL surface inventory | `reviews/task-97/001-qjl-surface-inventory/` approved by reviewer seq 01-03 | done |
| Distinct qjl counter kind/design | `reviews/task-97/002-qjl32-counter-kind-and-design/` plus reviewer-approved direction | done |
| Scalar qjl32 reference | `reviews/task-97/003-qjl32-scalar-reference/`; reviewer approved scalar reference | done |
| AVX2 local kernel | `reviews/task-97/004-qjl32-avx2-local/`; local qjl32 tests and Criterion evidence | local evidence done |
| IVF/SPIRE/HNSW AM registration | `reviews/task-97/005-qjl32-am-registration/`; local unit coverage and direct `turboquant_qjl` counters | local evidence done |
| NEON code slice | `reviews/task-97/006-qjl32-neon-local/`; x86 fallback tests pass; AArch64 cross check blocked by missing local C compiler | code landed; runtime evidence pending |
| SVE/SVE2 code slice | `reviews/task-97/007-qjl32-sve-dispatch-local/`; local qjl32 tests pass; SVE parity hook executes only on SVE hosts | code landed; Graviton evidence pending |
| Graviton 4 runtime dispatch | not run by user instruction | pending approval |
| Measured SVE vector length | not run by user instruction | pending approval |
| `isa=sve2` direct counter rows | not run by user instruction | pending approval |
| Per-AM closeout matrix | requires approved Graviton/runtime evidence and final closeout run | pending |

## Validation

No tests were run for this packet. It changes only task/index prose. Packet 007
contains the latest local behavior validation:

- `cargo test qjl32 --lib -- --color never`: 10 passed; 0 failed

## Notes

- Task 96 remains deferred by accepted stop condition in
  `reviews/task-96/001-surface-inventory-stop-condition/`.
- Task 94 local evidence is approved through packet 025 and remains pending
  Graviton 4 / final closeout evidence.
- This packet does not request Task 97 completion.
