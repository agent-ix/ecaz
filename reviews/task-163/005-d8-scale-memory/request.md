# Review request: D8 scale-memory and quality closure

## Scope

Please review the remaining Task 163 D8 measurement condition identified in
packets 003 and 004, and explicitly decide whether this supplies Task 179
acceptance criterion 1's required outside-reviewed bounded-spill closure.

Code checkpoints:

- `a675da3e3bb21088431d918b1fe9796686149d99` adds canonical build-backend
  RSS/HWM sampling and the encoded-completion high-water diagnostic.
- `cec8abba1770dc500a890c7ad57a932deae4c51c` persists PostgreSQL sharded-build
  NOTICE messages in loader artifacts and structures them in suite JSONL.

The underlying D8 implementation remains the accepted `de9d6fca3` plus
`c9b74c4f2` follow-up from packet 004. This packet closes its deliberately
deferred scale/quality measurement, not a new graph algorithm.

## Outcome

The exact-SHA canonical suite completed 5/5 steps, passed all 11 thresholds,
and has zero missing or stale artifacts.

At 10k/50k/100k, the complete build backend HWM is measured at 397,428 /
1,185,676 / 2,170,028 KiB. The durable sharded-build NOTICE and structured
JSONL record:

| Scale | Spill bytes | Completion high-water bytes | Stitch retained bytes |
| --- | ---: | ---: | ---: |
| 10k | 1,283,964 | 464,244 | 35,784 |
| 50k | 8,505,972 | 3,307,900 | 36,104 |
| 100k | 17,524,784 | 6,289,260 | 36,240 |

Thus stitch residency grows by only 456 bytes from 10k to 100k while the
spill grows 13.65x. The separate completion diagnostic answers the reviewer’s
worker-queue concern honestly; it is material but remains a flat-encoding
fraction of the spill rather than the old nested all-shard graph batch.

The same-config 10k pre-D8/current recall A/B is exactly equal at all five
search widths: 0.9950 / 0.9985 / 1.0000 / 1.0000 / 1.0000, each delta 0.0000
over 200 queries and 2,000 recall@10 trials.

## Why the evidence is decision-grade

- the release extension and runner are both built from exact remote-visible
  source `cec8abba1`;
- the suite records exact commands, corpus hashes, durations, build NOTICE
  values, `/proc` samples, and thresholds;
- every scale uses an isolated one-index-per-table prefix;
- server NOTICE values now appear in both the raw load logs and normalized
  `results.jsonl`, correcting the earlier packet's provenance gap; and
- the immutable pre-D8 baseline is cited in place rather than copied or
  reinterpreted.

The overall HWM is not mislabeled as stitch residency: it includes required
source vectors and output graph construction. FR-077-CON-4's bounded cursor +
one-group/prune surface is the `stitch_peak_retained_bytes` row.

## Requested decision

Please explicitly decide whether:

1. Task 163's deferred 10k/50k/100k NOTICE/RSS and 10k old-vs-new quality
   condition is closed; and
2. Task 179 AC-1 may cite packets 003–005 as outside-reviewed D8/FR-077-CON-4
   closure.

This packet does not close Task 179 itself. Its Task 172 acceptance and final
aggregate outside review remain separate gates.

## Validation

- focused shard suite: 16 passed, 0 failed;
- focused CLI parsing/expansion suites: 56 passed, 0 failed;
- `cargo check -p ecaz-cli`: pass with one pre-existing unrelated warning;
- release suite: 5/5 succeeded, 11/11 thresholds passed.
