# Packet 019 — scan-token RAII

Task bucket: `reviews/task-179/`; packet `019-scan-token-raii/`.
Head SHA: `230459ce6e6f64b9ae9c0854fa0ec867ee265353`.
Surface: coordinator shared scan-token registry only.

| File | Command | Key result |
|---|---|---|
| `validation.log` | commands listed in the artifact | Exact fingerprint-count unit test and strict clippy pass |

This is correctness evidence against the in-memory registry model. The existing
preloaded shared-memory drill is extended but remains in its dedicated ignored
lane until the physical CustomScan checkpoint exercises the preload path live.
No benchmark/corpus surface is involved.
