# Review Request: IVF TQ+ QJL/Gamma-Aware Mode

- Task: 89
- Packet: `reviews/task-89/002-ivf-tqplus-qjl-mode`
- Code commit: `a6b0970011fb64bde036c3f7a07321c0ab570885`
- Branch: `task-89-ivf-tqplus-profile`

## Summary

This checkpoint extends the IVF TQ+ experimental profile beyond the no-QJL 4-bit lane:

- Removes the TQ+ no-QJL guard in build/model validation.
- Encodes QJL-active TQ+ with calibrated MSE bytes plus existing QJL residual signs.
- Keeps residual gamma in the IVF posting gamma field for QJL-active dimensions.
- Appends a 4-byte candidate renormalization scalar to QJL-active TQ+ code bytes.
- Keeps no-QJL TQ+ storage width unchanged by reusing the IVF posting gamma field as candidate renormalization.
- Prepares QJL projection state for TQ+ queries and scores calibrated MSE + QJL residual terms before candidate renormalization.
- Adds core and IVF-layer tests for the QJL-active scalar/payload contract.
- Updates ADR-081 and Task 89 to record the experimental per-lane scalar shape.

## Validation

Artifacts are recorded in `artifacts/manifest.md`.

- `cargo check -p ecaz --lib --no-default-features --features pg18` passed.
- `cargo test -p ecaz --lib --no-default-features --features pg18 tqplus_` passed.

## What This Does Not Claim

This is still not Task 89 completion. It does not include:

- DBPedia 10k/50k/100k A/B benchmark evidence.
- Streaming-insert drift measurements.
- Cross-corpus measurements.
- Public shape gate decision.
- SPIRE/HNSW/DiskANN ports.

## Review Focus

- Is the QJL-active scalar layout acceptable for the experimental profile: posting gamma = residual gamma, appended f32 = candidate renorm?
- Does no-QJL remaining width-neutral preserve the intended packet 001 behavior?
- Is the TQ+ QJL scoring formula correct: calibrated MSE inverse/bias term + residual QJL term, then candidate renorm?
- Should the QJL-active appended scalar be generalized into a typed payload header before any benchmark gate, or is the current experimental metadata flag enough for Phase 3 measurement?
