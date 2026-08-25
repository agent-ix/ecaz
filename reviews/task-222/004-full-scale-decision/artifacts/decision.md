# Task 222 full-scale decision

Decision: **useful candidate; retain the production exact/fail-closed payload
projection implementation and request review closeout.**

| Scale | Recall control / candidate | Warm mean control -> candidate | p95 control -> candidate | Payload bytes/scan control -> candidate | Storage/arm |
| --- | --- | --- | --- | --- | ---: |
| 10k | 0.9990 / 0.9990 | 14.7 -> 8.76 ms (-40.41%) | 17.9 -> 9.37 ms | 121,624.72 -> 65.80 (-99.9459%) | 242,958,336 B |
| 50k | 0.9545 / 0.9545 | 16.8 -> 10.8 ms (-35.71%) | 19.1 -> 13.2 ms | 123,842.80 -> 67.00 (-99.9459%) | 1,243,553,792 B |
| 100k | 0.9290 / 0.9290 | 17.4 -> 11.6 ms (-33.33%) | 20.4 -> 13.9 ms | 123,103.44 -> 66.60 (-99.9459%) | 2,498,281,472 B |

Additional latency tails remain favorable: p99 is 18.4 -> 10.2 ms at 10k,
19.7 -> 13.5 ms at 50k, and 20.9 -> 14.0 ms at 100k.

The same remote-row counts are materialized in both arms: 6.58, 6.70, and
6.66 rows/scan at 10k, 50k, and 100k. The control requests four payload
columns per row in this release matrix; the candidate requests one (`id`), so
payload-column counts fall from 26.32/26.80/26.64 to 6.58/6.70/6.66 per scan.
The vector ordering operand is therefore proven excluded rather than hidden
inside an `{id, embedding}` mask.

| Scale | Owner payload SQL work/scan | Owner endpoint work/scan |
| --- | --- | --- |
| 10k | 7.490583 -> 0.446279 ms (-94.04%) | 7.773413 -> 0.722392 ms (-90.71%) |
| 50k | 7.597975 -> 0.448988 ms (-94.09%) | 7.876804 -> 0.724619 ms (-90.80%) |
| 100k | 7.683302 -> 0.514999 ms (-93.30%) | 7.977620 -> 0.817929 ms (-89.75%) |

## Identity and conformance

Control/candidate prediction SHA-256 values match within every scale:

- 10k: `801f6a0b83237047fea6ebd92cb1b85f07aa8dd80ee6dbd5c7877153e724fb6e`
- 50k: `1abc27ffe21b97f0513c35721e4e354fd2b379dbd7d7a7031fc009ac5f219e22`
- 100k: `228e17fbe4fa7480dced302f5b650721e6833d271503ad90b5d35b99d663eb0d`

Each A/B reports `build_shared=true` and `same_generation=true`; generation
identities are `0200c4cf...6d4a47`, `02003573...bb5a5`, and
`0200680a...7d7042` respectively. Every published topology has exactly the
source row count split over three owners, with `non_owned=0`, `orphans=0`, and
zero coordinator-resident unsharded bytes. Graph-side/raw-vector amplification
is 1.238533, 1.335307, and 1.353813, remaining below the registered 2.0 bound.
The payload projection adds no stored relation, so control/candidate storage is
identical. These facts classify both registered arms NFR-021 conforming and
make every scale's comparison NFR-022 same-generation admissible.

Packet 003 independently supplies the heavier 100k correctness matrix. Its
slightly larger row tier is expected because that fixture adds null/toast and
qual payload columns; this release matrix intentionally measures the standard
corpus schema.
