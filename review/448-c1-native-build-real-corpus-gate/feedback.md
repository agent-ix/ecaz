# Feedback: 448-c1-native-build-real-corpus-gate

Reviewed at branch `c1-task10065-native-hnsw-build-path` head `89a8c46`. Companion to 446 feedback; this file focuses on the real-corpus gate and harness drift.

## Verdict

Real-corpus numbers are healthy and consistent with the post-task16 50k TurboQuant surface:

- `(m=8, ef_search=40)  recall@10 = 0.886`
- `(m=8, ef_search=128) recall@10 = 0.930, exact@10 = 0.890`
- `(m=8, ef_search=200) recall@10 = 0.930`
- `(m=16, ef_search=200) recall@10 = 0.964`

These are strong evidence that the native BUILD replacement has not introduced a real-corpus recall regression.

## Answers to review questions

1. **Is the successful TurboQuant gate readout enough to treat real-corpus recall as provisionally satisfied?** Yes, for the TurboQuant surface. Caveat: this is *one* storage-format surface. If ADR-042 closure requires native-build recall parity on the other real-corpus surfaces (grouped heap-f32, pq_fastscan), run them before closeout — otherwise you're closing by inference, not proof, and that's the exact pattern we've pushed back on before.

2. **Summary-helper drift — follow-up harness work, or fold in before closeout?** Fold it in. Two reasons:

   - The working tree on this branch already contains uncommitted `src/lib.rs` changes (`SET LOCAL enable_indexscan/indexonlyscan/bitmapscan = off` around the exact-quantized baseline, plus a new `test_tqhnsw_external_summary_exact_baseline_multiidx` test) that look like the right fix for the grouped heap-f32 rerank selection reported here. Filing a packet that flags drift while the fix sits WIP on the same branch is confusing.
   - Either commit those changes as part of this slice with the harness-drift explanation, or stash/revert them. Don't ship the branch with WIP addressing the packet's own open question.

## Gap

Packet 446 reported hnsw_rs-vs-oracle source-graph recall (0.30 uniform / 0.2850 clustered / 0.6550 m=16-ef=200) but not native-vs-oracle on the same lanes. This packet's real-corpus evidence is code-graph / TurboQuant only. Before closeout add either:

- a native source-graph oracle lane in a follow-up packet, or
- a real-corpus run on a source-graph-backed surface.

Otherwise source-graph parity is asserted but not measured.
