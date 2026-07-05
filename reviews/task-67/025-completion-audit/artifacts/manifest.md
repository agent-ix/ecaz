# Task 67 Packet 025 Artifact Manifest

- Head SHA: `7773f1b3b2bf78e9f48f8c45e32524b05774874f`
- Task bucket: `reviews/task-67/`
- Packet path: `reviews/task-67/025-completion-audit/`
- Timestamp: `2026-05-30T07:29:40Z`
- Lane: Task 67 completion audit; no new benchmark run
- Fixture: not applicable for this packet; cites prior packet-local evidence
- Storage format: not applicable for this packet
- Rerank mode: not applicable for this packet
- Surface isolation: not applicable for this packet; no SQL run

## Artifacts

### `artifacts/git-status.log`

- Command:
  `git status --short --branch`
- Purpose: records current tracked cleanliness and known unrelated untracked
  artifact directories at audit time.

### `artifacts/relevant-commit-log.log`

- Command:
  `git log --oneline -30 -- src/quant/rabitq.rs crates/ecaz-cli/src/commands/bench/rabitq_kernel.rs src/am/ec_ivf/scan.rs`
- Purpose: records the recent Task 67 code/bench commits relevant to this
  audit.

### `artifacts/cloud-status.log`

- Command:
  `target/debug/ecaz cloud status --profile 10k-intel`
- Purpose: confirms the AWS Intel lane is paused at audit time.
- Key result:
  `state: paused`, `~$0.00/hr running`, retained storage `~$8.00/mo`.

## Cited Prior Evidence

- Packet 017:
  `reviews/task-67/017-intel-measurement-final/`
  - Real-10k Intel SQL measurement.
  - Recall preserved.
  - Primary bits=1 SQL wall-time speedup only 1.88x / 2.22x / 2.54x.
- Packet 020:
  `reviews/task-67/020-rabitq-kernel-bench/`
  - Approved by reviewer round 2.
  - AWS AVX-512 kernel evidence for bits1, bits4, bits8, bits8c3, bits8c4.
  - Key results include bits1 batch 5.59x, bits4 batch 9.02x, bits8 family
    batch about 11.8x, and bits8 single-dispatch at least 5.62x.
- Packet 021:
  `reviews/task-67/021-scratch-soa-bits1-measurement/`
  - Accepted as measurement.
  - Scratch-SoA plus auto-SIMD bits=1 SQL speedup versus packet 017 scalar:
    1.98x / 2.55x / 2.86x.
- Packet 022:
  `reviews/task-67/022-topk-frontier-bits1-measurement/`
  - Accepted with notes.
  - Best current bits=1 SQL-level result:
    2.11x / 2.52x / 3.07x versus packet 017 scalar.
- Packet 023:
  `reviews/task-67/023-rabitq8ls-kernel-bench/`
  - Accepted.
  - AWS AVX-512 `rabitq8ls` kernel row: 6.69x.
- Packet 024:
  `reviews/task-67/024-scratch-score-order-measurement/`
  - Negative evidence packet.
  - Score-ordering experiment regressed the packet 022 SQL-level result and
    was reverted in `db821441e`.

## Key Result Lines

Task 67 kernel-throughput gates:

| path | required | evidence | status |
| --- | ---: | ---: | --- |
| bits=1 batched path | >= 3x | 5.59x in packet 020 | pass |
| bits=4 kernel path | >= 5x per-slice target | 9.02x batch in packet 020 | pass |
| `rabitq8` | >= 4x headline / >= 5x per-kernel | 5.67x single, 11.76x batch in packet 020 | pass |
| `rabitq8ls` | >= 4x headline / >= 5x per-kernel | 6.69x in packet 023 | pass |
| `rabitq8c3` | >= 4x headline / >= 5x per-kernel | 5.75x single, 11.80x batch in packet 020 | pass |
| `rabitq8c4` | >= 4x headline / >= 5x per-kernel | 5.62x single, 11.77x batch in packet 020 | pass |

Task 67 strict SQL wall-time interpretation:

| path | evidence | status |
| --- | ---: | --- |
| bits=1 SQL at nprobe=16 | 2.11x in packet 022 | below 3x |
| bits=1 SQL at nprobe=32 | 2.52x in packet 022 | below 3x |
| bits=1 SQL at nprobe=64 | 3.07x in packet 022 | pass |

## Limitation

This packet does not add new benchmarks or code. It is a reviewable
requirement-to-evidence audit. The only unresolved point is the intended
scope of the Task 67 headline performance gate.
