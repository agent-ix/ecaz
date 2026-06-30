# Task 124 Packet 027: TQ speed closeout shelve

## Summary

Close Task 124 as **Shelve** under the corrected TurboQuant speed objective.

The reopened objective in packet `017` was not just to measure TQ. It required
continued TurboQuant-focused speed exploration, TQ-before/TQ-after evidence, and
honest separation between TQ-attributable wins and shared IVF/RaBitQ operating
point changes. That work is now complete enough to close:

- the in-engine TQ stage-2 path exists and is instrumented;
- the TQ scorer used by this path is full SIMD on the local ARM lane
  (`scalar_candidates=0`);
- multiple TQ-specific speed/materialization changes were built and measured;
- those TQ-specific speed changes were negative or exhausted;
- the `nprobe=60` speed observation was checked against f32/source and is not a
  TQ-attributable frontier win.

No product promotion is requested. No remaining Task 124 work is deferred.

## Decision

Shelve TQ stage-2 for this task. The durable result is:

1. TurboQuant can be wired as an IVF stage-2 reducer and can run on the SIMD
   scoring path.
2. The measured TQ component is not the remaining latency lever for the current
   design.
3. Narrow materialization/scoring hot-path optimizations did not improve speed.
4. The only observed speed win after reopen came from probing fewer shared IVF
   lists, and f32/source also holds recall at the same `nprobe=60` operating
   point.
5. TQ keeps the established storage penalty, about 4.5x f32/source index size
   at 100k in the final discriminator.

The `ec_ivf.tq_stage2_nprobe_cap` code may remain as a disabled-by-default
frontier operating-point knob, but it must not be described as a TurboQuant
speed optimization.

## Evidence Trail

Primary closeout evidence:

- `reviews/task-124/026-f32-vs-tq-nprobe60-discriminator/`: final
  reviewer-requested discriminator, 10k / 50k / 100k, 24 suite steps passed.
- `reviews/task-124/025-tq-selected-slab-vector/`: rejected TQ selected-payload
  lookup/materialization experiment, 10k / 50k / 100k, 18 suite steps passed.
- `reviews/task-124/020-tq-borrowed-score-buffer/`: rejected TQ borrowed score
  buffer experiment.
- `reviews/task-124/018-tq-selected-index-vector/`: rejected TQ selected index
  vector experiment.
- `reviews/task-124/017-speed-objective-correction/`: reopened the task under
  the correct TQ speed objective and corrected the premature packet 016 closeout.
- `reviews/task-124/019-phase6-evidence-correction/`: corrected the invalid
  local macOS `F_NOCACHE` cold-cache claim.

Earlier implementation and matrix evidence:

- `reviews/task-124/001-tq-stage2-engine-slice/`: in-engine TQ stage-2 path.
- `reviews/task-124/002-tq-stage2-attribution-counters/`: TQ attribution
  counters.
- `reviews/task-124/003-tq-stage2-ab-suite/` and
  `reviews/task-124/005-tq-final-width-sweep/`: 10k / 50k / 100k stage-2
  benchmark matrix and TQ final-width evidence.
- `reviews/task-124/011-tq-selected-payload-slab/` through
  `reviews/task-124/014-tq-direct-slot-rerank/`: structural TQ storage/layout
  attempts, measured and rejected or reverted.

## Final Discriminator Result

Packet `026` answered the reviewer condition from packet `023`: does TQ
stage-2 plus exact rerank allow `ec_ivf` to probe fewer coarse lists at equal
recall where f32/source cannot?

Result: **No.** At `nprobe=60`, f32/source also preserved recall at 10k, 50k,
and 100k. TQ was faster in that run, but the speed was not attributable to a
unique TQ frontier advantage, and TQ had lower recall at 50k.

| Scale | Variant | Recall@10 | NDCG@10 | p50 | p95 | p99 | ec_ivf index |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | f32/source | 1.0000 | 1.0000 | 1.22 ms | 1.38 ms | 1.43 ms | 2.9 MiB |
| 10k | TQ final15 | 1.0000 | 1.0000 | 1.13 ms | 1.28 ms | 1.37 ms | 10.9 MiB |
| 50k | f32/source | 1.0000 | 1.0000 | 4.48 ms | 5.32 ms | 5.83 ms | 11.6 MiB |
| 50k | TQ final15 | 0.9980 | 1.0000 | 4.23 ms | 4.47 ms | 4.54 ms | 50.9 MiB |
| 100k | f32/source | 1.0000 | 1.0000 | 9.46 ms | 9.76 ms | 9.92 ms | 22.5 MiB |
| 100k | TQ final15 | 1.0000 | 1.0000 | 8.77 ms | 9.01 ms | 9.22 ms | 100.8 MiB |

TQ scorer counters in packet `026`:

| Scale | TQ candidates | TQ scalar candidates | TQ elapsed | TQ ISA |
| --- | ---: | ---: | ---: | --- |
| 10k | 7,500 | 0 | 1.811008 ms | neon |
| 50k | 7,500 | 0 | 1.851708 ms | neon |
| 100k | 7,500 | 0 | 1.907458 ms | neon |

## Acceptance Audit

| Requirement / phase | Outcome |
| --- | --- |
| Phase 0 SIMD audit | Satisfied for the TQ path used by this task. Packet `002` and later benchmark counters show `scalar_candidates=0`; packet `026` repeats this at 10k / 50k / 100k. |
| Phase 1 in-engine stage-2 | Satisfied by packet `001`. |
| Phase 2 payload/storage exploration | Satisfied for a shelve decision. Packets `011` through `014` and later speed packets explored selected slabs, top-k fusion, compact group headers, direct-slot rerank, and selected lookup alternatives; measured results did not justify landing those changes. |
| Phase 3 counters | Satisfied by packet `002` and subsequent suite artifacts. |
| Phase 4 recall/correctness | Satisfied for closeout. TQ can match recall at selected points, but packet `026` shows it does not uniquely preserve recall under `nprobe=60`; at 50k, f32/source was `1.0000` and TQ was `0.9980`. |
| Phase 5 10k / 50k / 100k matrix | Satisfied. Packet `026` is the final 10k / 50k / 100k discriminator; packets `003`, `005`, `023`, `024`, and `025` provide earlier matrices. |
| Phase 6 IO-sensitive validation | Corrected. Packet `019` records that packet `015` was not controlled cold-cache evidence. No product latency claim depends on packet `015`. |
| Reopened TQ speed objective | Satisfied for a shelve. Real TQ speed changes were built and measured in packets `018`, `020`, and `025`; they were negative. Packet `026` rules out the nprobe60 frontier result as a TQ-attributable win. |

## Landing Recommendation

- Do not promote TQ stage-2 as a product path from Task 124.
- Do not claim `nprobe=60` as a TurboQuant optimization.
- Keep only the code that is already justified as infrastructure or an opt-in
  diagnostic/operating-point knob.
- Future TQ work should be a new task with a changed premise, such as a storage
  layout redesign or different quantizer, not another narrow micro-optimization
  pass over this same stage-2 shape.
