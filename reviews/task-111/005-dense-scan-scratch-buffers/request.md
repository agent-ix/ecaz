# Task 111 Review Request: Dense Scan Scratch Buffers

## Scope

This packet reviews code checkpoint `5e9e9fecb00` (`Task 111: reuse dense scan scratch buffers`).

It addresses the dense scan allocation gap raised in packet 002 feedback: dense block scan no longer creates a fresh gamma vector and score vector for each block.

## Change

- Adds `IvfDensePostingBlockScratch` as scan-opaque-owned reusable state.
- Frees the dense scratch buffer from `amendscan` with the rest of scan-owned state.
- Adds `IvfDensePostingBlockRef::copy_gammas_to` so scan can decode dense gamma values into reusable storage.
- Routes dense batch scoring and scalar fallback through the reusable gamma/score buffers.
- Adds a unit test that verifies dense scratch reload clears scores and keeps gamma/score capacity.

## Validation

Packet-local artifacts:

- `artifacts/cargo-check-lib.log`
- `artifacts/cargo-test-dense-posting.log`

Results:

```text
cargo check -q --lib
```

exited successfully.

```text
cargo test -q dense_posting --lib
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 2102 filtered out
```

## Remaining Task 111 Work

- Run the required `ecaz bench suite` evidence for TurboQuant and RaBitQ latency/recall/storage/build-time comparison before any promote/iterate/abandon recommendation.
- Add any benchmark-driven adjustments needed by that evidence.
