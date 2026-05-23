# Task 51 Review Request: RaBitQ8 Sidecar Clip Tuning

## Scope

This packet follows up on the Task 51 closeout finding that the best measured `rabitq8` sidecar path was fast but recall-limited.

Commits under review:

- `b09c8d75c` Add RaBitQ sidecar clip tuning variants
- `bf84190ad` Record local RaBitQ sidecar recall lift

The implementation keeps existing RaBitQ defaults intact. The new constructor accepts an explicit scalar quantization clip radius, and the sidecar measurement harness exposes explicit q8 variants:

- `rabitq8`: existing 2 sigma clip
- `rabitq8ls`: 2 sigma with least-squares scoring, negative-control result
- `rabitq8c3`: 3 sigma clip
- `rabitq8c4`: 4 sigma clip

## Evidence

Benchmark packets:

- `benchmarks/task51-local-rabitq8-sidecar-recall-sweep/`
- `benchmarks/task51-local-rabitq8ls-sidecar/`

Packet-local manifest:

- `reviews/task-51/024-rabitq8-sidecar-clip-tuning/artifacts/manifest.md`

Validation:

- `cargo test -p ecaz-cli --no-default-features sidecar`: 7 passed
- `cargo build -p ecaz-cli --no-default-features`: passed
- `ecaz bench suite status`: completed 1, failed 0, skipped 0 for the q8 scoring variant suite

## Results

Candidate frontier sweep showed recall did not improve with larger frontiers:

| candidate_k | recall@10 | sidecar p50 |
| ---: | ---: | ---: |
| 50 | 0.9480 | 0.929 ms |
| 100 | 0.9480 | 1.674 ms |
| 200 | 0.9480 | 2.704 ms |

Scoring variant sweep found the useful lever:

| variant | read mode | recall@10 | sidecar p50 | bytes/vector |
| --- | --- | ---: | ---: | ---: |
| `rabitq8` | `tid-sorted` | 0.9480 | 2.013 ms | 1548 |
| `rabitq8ls` | `tid-sorted` | 0.9490 | 2.005 ms | 1548 |
| `rabitq8c3` | `tid-sorted` | 0.9810 | 2.049 ms | 1548 |
| `rabitq8c4` | `tid-sorted` | 0.9950 | 2.387 ms | 1548 |

## Reviewer Focus

- Check that the default RaBitQ code path still uses the existing 2 sigma clip.
- Check that the sidecar harness variants are measurement-only and do not silently change production scan behavior.
- Confirm the benchmark evidence is enough to justify taking `rabitq8c4` forward into product sidecar design.
