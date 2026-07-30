# Arm-blind storage retrospective

## Classification rule

The old `physical_benchmark_storage` row measured the immutable physical
generation before entering the variant loop and then repeated those values for
each arm. It could answer “did these arms reuse the same base generation?” It
could not answer “does each arm have equal total resident state?”

This sweep therefore uses:

- **SOUND:** the evidence can represent the storage property asserted.
- **QUALIFIED:** the statement remains supportable only when narrowed to the
  immutable generation or another explicitly bounded/query-only surface. It is
  not NFR-021/NFR-022 conformance evidence.
- **INVALID:** the decision treated an arm-blind row as total resident storage
  even though an arm added an unbounded derived relation.
- **N/A:** no arm-storage equivalence claim or no ec_distann packet exists.

This classification does not re-open recall or latency findings by itself.

## Packet-level claims

| Task / packet | Claim and citation | Reclassification | Reason |
| --- | --- | --- | --- |
| 163 / `001-m1-stitch-ab` | “Storage & latency identical” (`request.md:40`) | **QUALIFIED** | Mono and stitch retain one persistent record per vector. The row supports base-generation identity, not equality of all possible arm-local resident state. |
| 179 / `035-physical-epoch-cache` | Storage differences are page-layout noise (`verdict.md:18`) | **QUALIFIED** | The epoch cache is bounded/query-side. Preserve the generation claim; do not use the row as an NFR-021 verdict. |
| 179 / `038-head-cap-sensitivity` | The packet explicitly says the standard metric excludes the shared coordinator head (`verdict.md:17`) | **SOUND CAVEAT** | This packet already states the metric boundary and does not infer total resident equality. |
| 179 / `048-persisted-head-ab` | Physical storage identical at 10k/100k, page-level difference at 50k (`request.md:40`) | **QUALIFIED** | Valid for the shared persistent generation only; coordinator head state was outside the old row. |
| 179 / `052-prompt-cancellation-ab` | Physical storage identical/page-level differences (`request.md:37`) | **QUALIFIED** | Query cancellation does not add an O(N) structure; the claim is generation-scoped. |
| 179 / `068-transport-latency-isolation` | Storage unchanged (`request.md:32`) | **QUALIFIED** | Transport/query-only comparison; no derived persistent state, but the old row cannot prove total resident equality. |
| 179 / `069-all-findings-signoff` and `071-all-findings-final-signoff` | Storage unchanged apart from 32 KiB; generation storage identical (`disposition.md:40`; `071.../disposition.md:42,50`) | **QUALIFIED** | Keep the generation/page statement, not an NFR-021 conclusion. |
| 179 / `070-system-column-latency-isolation` | Generation storage unchanged/page-level difference (`request.md:31-33`) | **QUALIFIED** | System-column projection is query-only and did not add an O(N) relation. |
| 181 / `006-decision-correction` | Physical storage effectively unchanged (`artifacts/manifest.md:29`) | **QUALIFIED** | The evidence compares persistent generations and page rounding, not all coordinator state. |
| 184 / `003-isolated-candidate`, `004-full-scale-decision` | Storage/construction shared and identical (`003.../request.md:60`; `004.../request.md:38`) | **QUALIFIED** | Lazy materialization is query-time and bounded. Shared-generation identity is valid; resident-state equivalence was not measured. |
| 188 / `002-search-graph-attribution`, `004-full-scale-decision`, `005-batch10-reconfirmation`, `008-final-finding-disposition` | Storage/head unchanged or identical (`002.../request.md:21`; `004.../request.md:31`; `005.../request.md:25`; `008.../request.md:23`) | **QUALIFIED** | Beam width/query scheduling reuse one immutable generation. The storage delta can be described as zero for that generation, not as a mechanical NFR-021 result. |
| 191 / `003-production-full-scale`, `004-closeout` | Storage/construction identical because the A/B reuses one generation (`003.../request.md:42`; `004.../request.md:49`) | **QUALIFIED** | This rationale is correct for the explicitly shared immutable generation. |
| 192 / `005-paired-cache-ab`, `006-epoch-safety`, `007-full-scale-decision` | Physical storage shared/unchanged or identical (`005.../request.md:39`; `006.../artifacts/manifest.md:6`; `007.../request.md:38`) | **QUALIFIED** | The schema cache is bounded by relation/projection identity. Preserve the scoped claim, not a total-state inference. |
| 193 / `005-owner-plan-candidate` | Identical generations; storage unchanged (`request.md:20,26`) | **QUALIFIED** | Four-plan LRU is explicitly bounded and owner-local. |
| 194 / `007-fixed-work-candidate` | Identical storage on one shared generation (`request.md:27,35`) | **QUALIFIED** | The arm changes query work only. This does not repair the separate Task 203 traversal-regime defect. |
| 198 / `004-isolated-100k`, `005-full-scale-decision` | Owner and coordinator-replica rows were identical | **INVALID** | The old row was repeated by construction while the replica arm added a 1,659,518,976-byte unsharded coordinator relation. See Task 203 packet 001 Defect 4b and manifest citation index. |
| 199 / `003-release-matrix-and-decision` | “Storage is identical between arms at every scale” (`request.md:20`) | **INVALID** | Same omitted O(N) relation as Task 198; the statement was used in a promotion decision after reviewer objection. |
| 204 / `001-arm-fidelity` | Owner and replica arms have different cluster/per-node resident bytes (`request.md`; reviewer feedback seq 01) | **SOUND** | Storage is emitted inside the arm loop; the 1,659,518,976-byte coordinator relation appears on the coordinator node and changes the cluster ratio. |
| 205 / `003-ab` | Owner control and candidate have equal corrected per-arm storage at 10k/50k/100k | **SOUND MEASUREMENT; SUPERSEDED JUDGEMENT** | Both arms are owner-traversal and the corrected rows show no derived relation. The packet's old raw fixed-roster growth judgement is superseded by Task 208 packet 001's normalized NFR-021 rule; the Task 205 performance disposition still requires its corrected A/B rerun. |

No other audited packet makes a decision-bearing “identical/unchanged arm
storage” claim. Mentions of an unchanged storage *format*, a shared fixture, or
an unchanged source-code storage API were not treated as measurement claims.

## Task-level T4 closure

This table replaces every Task 203 `T4 = ?` with an explicit state. “Qualified”
means the task may cite base-generation identity but may not cite the old row as
total resident-state or NFR-021 evidence.

| Task | T4 closure | Basis |
| --- | --- | --- |
| 161 | **N/A** | Spec authoring; no benchmark storage-equivalence disposition. |
| 162 | **N/A** | Single-node kill-check; no arm-local persistent-state decision. |
| 163 | **QUALIFIED** | Stitch A/B claim is base-generation-scoped. |
| 164 | **N/A / lane inadmissible** | Replicated-serving control is already self-declared non-decision-bearing. |
| 165 | **N/A / lane inadmissible** | Replicated-serving control; no valid storage-equivalence disposition. |
| 166 | **N/A / lane inadmissible** | Single-instance control; no valid distributed storage decision. |
| 167 | **N/A** | DML work without a storage-equivalence disposition. |
| 172 | **SOUND SHELVING** | Historical run emitted the amplification row; its replicated fixture was correctly rejected as a gate. |
| 179 | **QUALIFIED** | Several generation/page claims are valid only at that scope; packet 038 already records the coordinator-state exclusion. |
| 180 | **N/A** | Uses a shared generation but does not decide equal total resident state. |
| 181 | **QUALIFIED** | Persistent-generation/page comparison only. |
| 182 | **N/A** | No decision-bearing arm-storage equivalence claim. |
| 183 | **N/A** | No decision-bearing arm-storage equivalence claim. |
| 184 | **QUALIFIED** | Query-time bounded lazy materialization; shared generation. |
| 185 | **N/A** | No measured decision packet. |
| 186 | **N/A — no packet** | Proposed task; no committed review bucket. |
| 187 | **N/A — no packet** | No committed review bucket. |
| 188 | **QUALIFIED** | Beam/query arms share the immutable generation. |
| 189 | **N/A — no packet** | Dormant proposal; no committed review bucket. |
| 190 | **INVALID ARCHITECTURE / N/A CLAIM** | No arm-equivalence measurement claim, but the selected O(N)-per-coordinator architecture is NFR-021-inadmissible. |
| 191 | **QUALIFIED** | Query-time materialization arms share the immutable generation. |
| 192 | **QUALIFIED** | Bounded schema cache; shared generation. |
| 193 | **QUALIFIED** | Bounded owner-plan LRU; shared generation. |
| 194 | **QUALIFIED** | Query-work arm; shared generation. |
| 195 | **N/A** | Bounded owner cache, no storage-equivalence decision. |
| 196 | **N/A** | Scan-local prefix reuse, no persistent-state decision. |
| 197 | **SOUND** | Positive example that emitted `cluster_index_space_amplification`. |
| 198 | **INVALID** | Arm-blind row omitted the candidate's O(N) coordinator relation. |
| 199 | **INVALID** | Same omitted relation used in the promotion decision. |
| 200 | **N/A** | Benchmark memory-retention fix; no production resident-state arm. |
| 201 (ec_distann) | **INVALID BY INHERITANCE / no packet** | Proposed frozen control contains the non-conforming replica. The existing `reviews/task-201/` belongs to the unrelated double-allocated Task 38 follow-up. |
| 202 | **N/A — no packet** | Cross-ISA portability proposal; no storage arm. |
| 203 | **AUDIT** | Established the arm-blind defect and opened this sweep. |
| 204 | **SOUND** | Corrected per-arm/per-node/relation accounting. |
| 205 | **SOUND MEASUREMENT; OPEN DISPOSITION** | Corrected 10k/50k/100k owner-arm storage exists; old raw-growth judgement and inert performance A/B cannot close the task. |

## Ratio-row audit

- Task 172 packet 001 contains the historical
  `cluster_index_space_amplification` row
  (`artifacts/results.jsonl:8`; manifest line 41). Its shelving remains sound.
- Task 197 packet 001 contains the positive preflight ratio row.
- Tasks 198/199 contain no structured ratio row. Their hand-computed prose
  ratios do not satisfy the row-presence requirement and cannot see the replica
  relation.
- Task 204 packet 001 contains corrected per-arm
  `physical_benchmark_storage_ratio` rows, including owner `1.351147` and
  coordinator-replica `4.052187`.
- Task 205 packet 003 contains corrected per-arm ratio/node/relation rows at
  10k/50k/100k. Task 208 packet 001 adds mandatory row-presence enforcement and
  the normalized cross-scale conformance judgement.

## Final disposition

The sweep changes no production code and closes no outside review. It removes
the ambiguity behind Task 203's `T4 = ?` cells:

- shared-generation identity is useful evidence when named precisely;
- it is not a proxy for total resident state;
- Tasks 198/199 remain the only historical decisions in this corpus where the
  omitted state changes the architectural admissibility verdict;
- future decisions must consume the corrected per-arm rows plus Task 208's
  pre-registered conformance result.
