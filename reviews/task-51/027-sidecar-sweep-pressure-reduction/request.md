# Review Request: Sidecar Sweep Pressure Reduction

- Code commit: `0429af2ab72fcb6577daf22e67f520c6b3f08230`
- Benchmark packet: `benchmarks/task51-aws-rabitq8-sidecar-full-sweep`
- Scope: reduce AWS sidecar sweep memory and rebuild pressure without changing IVF/RaBitQ index behavior

## Change

The sidecar rerank harness now builds, loads, and scores one sidecar variant at a time instead of retaining all requested sidecar encodings simultaneously. Candidate frontiers are still collected once per sweep value and reused across variants, preserving the intended comparison while reducing peak memory during the new `rabitq8`, `rabitq8ls`, `rabitq8c3`, and `rabitq8c4` AWS sweep.

The AWS suite config also stops forcing sidecar measurement table rebuilds. Existing complete sidecar tables are reused; absent or partial tables are still populated by the harness because the row-count guard remains in place.

## Validation

- `artifacts/cargo-build-release-ecaz-cli.log`: release build passed.
- `artifacts/cargo-test-ecaz-cli-sidecar.log`: focused sidecar tests passed, `7 passed; 0 failed`.

## Notes

This packet is for the local pressure-reduction code/config change only. The active AWS benchmark attempt that started before this commit does not include the one-variant-at-a-time harness change; if that run fails or exceeds the spend cap, the pushed commit provides the next lower-pressure AWS path.
