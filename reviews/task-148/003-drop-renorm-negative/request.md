# Task 148 Packet 003: Drop Measured-Negative Length Renorm

## Scope

This packet removes the active Slice 2 length-renormalization product change after packet 002 measured it as a negative for the pure TQ no-rerank cell.

Code checkpoints:

- `da7ede88a` reverts `9ded20145` (`Allow TQ sidecar scoring without gamma`).
- `2de14b389` reverts `a3bcb13d` (`Apply TQ no-QJL length renormalization`).

The packet 002 measurement evidence remains intact under `reviews/task-148/002-length-renorm-ab/`.

## Rationale

Packet 002 showed:

- Pure TQ 100k recall improved only about `+0.62` to `+0.63 pp`.
- Pure TQ latency regressed about `5-6x` at the required nprobe 32/40 checks.
- Stage2 was unchanged because persisted sidecar scoring does not carry gamma.

Leaving that correction enabled would violate the Task 148 latency-neutral gate and would stack a known-regressing change onto Slice 3. This revert restores the pre-renorm no-QJL scoring behavior before the codebook calibration slice.

## Validation

No test run was performed for this cleanup checkpoint. The code changes are exact git reverts of the two Slice 2 implementation commits, and packet 002 contains the measurement evidence motivating the revert.

No push was performed per handoff.
