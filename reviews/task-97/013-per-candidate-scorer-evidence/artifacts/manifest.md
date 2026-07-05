# Task 97 Packet 013 Artifact Manifest

- head SHA: `8742d7f2bca185262889038628ead2756c120da9`
- task bucket: `reviews/task-97/013-per-candidate-scorer-evidence`
- lane: Task 97 TurboQuant QJL block kernel
- fixture: local Criterion, QJL-active `dim=1024,bits=4,seed=42`
- storage format: packed TurboQuant code bytes `[mse_packed][qjl_packed]`
- rerank / exact mode: production QJL (`MseLutQjl`)
- host ISA: local x86_64 production dispatch
- AWS / CI: not run

## Scope

This packet addresses part of packet 004 F1 by making the current production per-candidate QJL scorer measurable at the Task 97 fixture dimension.

Code checkpoint `8742d7f2bca185262889038628ead2756c120da9` adds `1024d/4-bit` to the existing Criterion `quant/score_ip_from_parts` group in `benches/criterion/quant_score.rs`.

## Commands

- Formatting:
  `cargo fmt --check`
- Diff hygiene:
  `git diff --check`
- Local per-candidate Criterion row:
  `cargo bench --features bench --bench quant_score 'quant/score_ip_from_parts/d1024_b4' -- --sample-size 10 --warm-up-time 1 --measurement-time 2`

The first attempt used `script` as the packet logger and stalled with an empty file; I stopped the stale local wrapper and reran the same command with `tee`. The successful packet evidence is `local-cargo-bench-score-ip-from-parts-d1024.log`.

## Primary Artifacts

- `local-cargo-bench-score-ip-from-parts-d1024.log`

## Key Result Lines

Current-head production per-candidate scorer:

- benchmark: `quant/score_ip_from_parts/d1024_b4/1024`
- time: `[874.53 ns 887.34 ns 904.33 ns]`
- throughput: `[1.1058 Melem/s 1.1270 Melem/s 1.1435 Melem/s]`

## F1 Status

This packet supplies the durable current-head Criterion row for `score_ip_from_parts` at `dim=1024`.

It does not claim the full packet 004 F1 is closed. A complete F1 closure still needs either:

- an old-vs-new Criterion comparison against the pre-`b0efa19d9` multi-accumulator production scorer using the same `d1024_b4` fixture; or
- a reviewer-accepted disposition that the current per-candidate row plus packet 011's local scoring-ladder evidence is sufficient for the Task 97 stop-condition / optimization decision.

## Validation

- `cargo fmt --check`: passed
- `git diff --check`: passed
- Local Criterion row above: passed
- No GitHub CI or AWS runs were used.
