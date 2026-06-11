# Task 97 Packet 002: qjl32 Counter Kind and Design

This packet follows the approved Phase 0 QJL inventory and addresses reviewer
feedback seq 01-03.

## Code Change

- Adds `QuantCodecKind::TurboQuantQjl` with label `turboquant_qjl`.
- Extends the block-kernel counter matrix from 4 to 5 quant kinds.
- Adds a focused counter test proving QJL rows emit distinct direct rows and do
  not increment Task 87 `lut32_*` compatibility fields.

No qjl32 kernel module is implemented yet.

## Design

The qjl32 design is packeted in `artifacts/design.md`.

Key decisions:

- `src/quant/qjl32/` is a separate ADR-076 family, not a lut32 mode branch.
- Candidate side data stays `CandidateMeta::Gamma`; residual signs remain in
  current code payload bytes.
- Initial production scope is canonical `bits=4`, non-1536 dimensions, where
  `ExactScoreMode::MseLutQjl` is reachable.
- AM registration targets IVF, SPIRE, and HNSW; DiskANN remains out of scope.
- Local measurement fixture is synthetic `dim=1024,bits=4,seed=42`.

## Validation

- `cargo test candidate_batch --lib -- --color never`
  - 15 passed
  - log: `artifacts/local-cargo-test-candidate-batch.log`

No CI, AWS, or benchmarks were run.

## Review Request

Please review the new counter kind and the qjl32 design. If accepted, the next
packet will implement the scalar qjl32 reference and bit-exact parity tests
against the existing pre-kernel scorer.
