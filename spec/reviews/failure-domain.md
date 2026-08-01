---
id: SR-002
title: "Failure-Domain Analysis of the Tasks 211-214 DistANN Spec Round"
type: SpecReview
analysis: failure-domain
scope: "FR-088, FR-089, FR-090 (new mechanisms, primary), FR-080 and FR-086 (rewritten this round); context FR-085, FR-082, FR-079, NFR-021; branch task-203-ec-distann-conformance @ 8165ff2d8; grounded against reviews/task-214/001-drift-inventory/artifacts/"
review_set: subset
---
# SR-002: Failure-Domain Analysis — Tasks 211-214 Spec Round

## Summary

This round examines the three new read-path mechanisms — FR-088 head scaling
law, FR-089 crown cache, FR-090 fused head hop — plus the rewritten FR-080
(sharded head) and FR-086 (gateway copies), for unstated failure modes,
identity confusion, purity gaps, and topological edge cases (quoin
spec-failure-domain-analysis method). The prior round's SR-002 content
(ec_distann base batch, reconciled at b19551e21, all findings dispositioned)
stands in git history; this document supersedes it for the current scope.

Two high findings: FR-089's width-pruning clause contradicts its own
identical-results acceptance criterion and, combined with lazy partial
population, permits a silent set-level narrowing of the head search that no
per-entry miss/fallback ever triggers; and FR-090 never defines the fused
request's wire shape — FR-079 has no seed-candidate field, and whether
crown-ranked candidates are expanded (neighbors returned) or only
exact-scored is ambiguous, so the round-trip saving, fan width, threshold
application, and NFR-019 accounting are all asserted over an undefined
request. The remaining findings cluster on: law-resolution edge cases
(C vs sample_count identity at C>N, clamp with floor>ceiling, NaN/negative
rate, defaulted-vs-explicit cap precedence, float determinism across ISAs,
no slot in the frozen FR-082 manifest layouts for the sizing attestation);
lazy-population trust-boundary gaps (population-RPC failure policy,
"populated" undefined, mid-fused-request failure re-entry); attestation and
identity gaps (crown selection digest depends on a session GUC the build
cannot attest, replica attested-but-unservable behavior, gateway subset
nondeterminism); and within-epoch tombstone staleness in both code caches.

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-001 | high | FR-089 width pruning contradicts FR-089-AC-1: fanning the head search only to "promising" shard holders can change the merged seed set (and thus results), and under lazy partial population a not-yet-populated shard scores as unpromising and is pruned — a set-level narrowing that no per-entry "crown miss" fallback ever detects; silent substitution risk of the kind NFR-021 clause 4 exists to forbid | FR-089, FR-089-AC-1, FR-080, NFR-021 |
| FND-002 | high | FR-090 never defines the fused request's wire shape: FR-079's endpoint has no seed-candidate field and FR-090 amends no signature; whether crown-ranked seed candidates are expanded (neighbors returned — the RTT actually saved) or only exact-scored is ambiguous, and the fused first round's fan width (seed_count? BW? crown-rank cutoff?) and its NFR-019 BW×H accounting are unstated — FR-090-AC-1/AC-2 assert order and threshold semantics over an undefined request | FR-090, FR-079, FR-080, NFR-019 |
| FND-003 | medium | FR-090 mid-fused-request failure is unhandled: fallback is specified only as a pre-request decision (crown off/unpopulated/miss); an epoch-mismatch during the fused first expansion restarts "from the head index" (FR-082) without saying whether the retry re-enters fused or unfused, and a non-retriable owner failure aborts the query (FR-079) so the promised "correct slow path" is unreachable mid-scan; what partial state (crown-ranked candidates, partially returned seed distances) is discarded and whether the attempt budget resets is unstated | FR-090, FR-082, FR-079, NFR-019 |
| FND-004 | medium | Population trust boundary unstated for both code caches: FR-089 ("populated lazily by bounded batch RPCs") and FR-086 ("populated per epoch via bounded owner batch RPCs") never define when population runs relative to a scan, the failure policy of a population RPC (strict abort vs resilient degrade-to-fallback), or what "populated" means for a lazily filled cache — yet FR-090's entry gate ("when the crown is populated for the pinned epoch") keys on exactly that undefined predicate | FR-089, FR-086, FR-090 |
| FND-005 | medium | FR-089 selection-digest attestation has an identity gap: crown selection is "sized to capacity" where capacity is a per-session GUC, so a build/manifest-time attested digest cannot exist for all capacities; who computes and attests the digest, when, and under what key (epoch × capacity? per backend?) is unstated, and per-backend GUC divergence makes "the" crown of an epoch ill-defined while the lifecycle clause keys the crown by epoch fingerprint only | FR-089, FR-089-CON-2, FR-088 |
| FND-006 | medium | FR-088 conflates resolved capacity C with actual head size: with floor 4096 and N < floor, C > N and the selected membership has sample_count < C; the manifest attests C while the membership blob decodes sample_count (FR-080-AC-8) — which number governs trained-policy validity (C = 4096), crown capacity-vs-C comparisons, and the frozen 16..=1,048,576 validity domain is unstated | FR-088, FR-088-CON-1, FR-080-AC-8, FR-089 |
| FND-007 | medium | FR-088 law-resolution misconfiguration domain undefined: floor > ceiling makes clamp undefined (panics in the obvious implementation), floor below 16 or ceiling above 1,048,576 escapes the frozen validity domain CON-1 asserts, and rate NaN (fails `> 0`, silently disabling the law), negative, or infinite are never rejected; no validation point (reloption set vs T2 build) or error class is named | FR-088, FR-088-CON-1 |
| FND-008 | medium | FR-088 precedence between law and explicit cap is undecidable in the stated config model: `head_index_cap` is a pre-existing reloption with a default, and reloptions do not distinguish "explicitly set" from "defaulted", so a build with rate > 0 and an untouched cap cannot tell which branch ("explicit cap takes precedence") applies; the override attestation bit then attests an unobservable distinction | FR-088, FR-088-AC-3, FR-080-CON-2 |
| FND-009 | medium | FR-088's sizing attestation has no home in the digest chain: the FR-082 v2 manifest and its v1/v2 build_options subrecords are frozen fixed-layout wire formats (30 bytes, `head_index_cap u32`) with no slot for rate, floor, ceiling, N, or the override flag; whether attestation requires a manifest v3 / options v3 (and what old-epoch decode does) is unstated, yet FR-088-AC-2 requires tamper-evidence through that chain | FR-088, FR-088-AC-2, FR-082 |
| FND-010 | medium | FR-088 determinism claim rests on unstated float semantics: `ceil(rate × N)` near an integer boundary can resolve differently across float widths, rounding modes, and ISAs (Intel / Graviton / M5 are all supported lanes), and the canonical encoding of `rate` inside the attestation (bit pattern vs decimal) is unstated — "identical build inputs yield identical C" and CON-2's replay-identical attestation need both pinned | FR-088, FR-088-CON-2, FR-082 |
| FND-011 | medium | FR-080 replica attested-but-unservable behavior undefined: attestation proves import completed, not that the copy remains readable; when routing selects a replica whose imported shard fails to serve (corruption, dropped copy table rows), whether the scan retries the shard owner or errors is unstated; replica copy-table cleanup at epoch retire/reclaim and for nodes removed from the successor roster is also unstated (stale attested copies accumulate outside the FR-082 reclaim inventory) | FR-080, FR-082, NFR-021 |
| FND-012 | low | FR-086 gateway subset is selection-nondeterministic: "refusal-bounded subset" makes membership depend on population batch order, which is unstated — unlike the crown's attested deterministic selection — so per-backend gateway sets diverge and response-byte A/B metrics (FR-086-AC-5) are not reproducible across backends or runs; results are unaffected (AC-1) but the measured quantity is | FR-086, FR-086-AC-5, FR-089 |
| FND-013 | low | Within-epoch tombstone staleness in both code caches: head membership is frozen (D10) but landmarks can be tombstoned mid-epoch (FR-083); the rebuild-only crown keeps ranking a tombstoned landmark as a promising seed and the gateway entry caches a tombstone flag whose authority is the owner anyway (purpose and staleness of the cached flag unstated); whether head search excludes tombstoned landmarks — and whether fused and unfused paths agree on that — is unstated, a seed-set-equivalence hazard for FR-090-AC-4 | FR-089, FR-086, FR-090, FR-083, FR-085 |
| FND-014 | low | FR-088 leaves N's composition unstated: "cumulative captured record count at T2 seal" does not say whether tombstoned and replaced records captured by the build snapshot count toward N, so two builds over the same live set with different DML history can resolve different C while both claim the law | FR-088, FR-078, FR-083 |
| FND-015 | medium | FR-090 exact seed policy is structurally unachievable when crown ⊂ head: with capacity < C the coordinator selects seeds from a coarser universe than the unfused owner-shard search, so reproducing the unfused seed set exactly is impossible in general; the condition under which the exact policy (and the fixture seed-digest check) is claimable — capacity ≥ C? crown = full head? — is unstated, leaving AC-4's dichotomy without a decidable trigger | FR-090, FR-090-AC-4, FR-089, FR-088 |

## Resolutions (same session, post-review)

- FND-001 resolved: FR-089 width pruning is now an explicit measured arm —
  dedicated GUC, population-complete precondition (no pruning of shards the
  crown does not fully hold), labeled seed-set change excluded from AC-1.
- FND-002 resolved: FR-090 defines the fused request as an ordinary FR-079
  expansion whose requested vec_ids are the crown-ranked seed candidates
  (no wire extension), bounded by seed_count on round 1, with the
  NFR-019 accounting stated (`seed_count + BW × (H − 1)`).
- FND-003 resolved: FR-090 mid-request failure clause added — epoch
  mismatch consumes the single retry and re-enters unfused with crown
  state discarded; non-retriable owner failure aborts per FR-079.
- FND-004 resolved: FR-089 population runs at scan open before serving,
  degrades resiliently on RPC failure, and "populated" is defined as the
  population-complete predicate FR-090 consumes; FR-086 subset selection
  made deterministic (membership-order prefix).
- FND-005 resolved: FR-089 selection digest computed by the populating
  backend under the identity (epoch_fingerprint, capacity); CON-2 amended.
- FND-006 resolved: FR-088 attests both resolved C and sample_count and
  names which governs what.
- FND-007 resolved: FR-088 validation clause added (EC_HEAD_SIZING at T2
  for non-finite/negative rate, floor > ceiling, out-of-domain bounds).
- FND-008 resolved: precedence is rate-only (rate = 0 ⇒ explicit cap);
  the unobservable "explicitly set" distinction is removed; AC-3 rewritten.
- FND-009 resolved: attestation carrier named — build-options v3 subrecord
  (FR-078 wire family) with the v1/v2 decode rule.
- FND-010 resolved: arithmetic pinned (one f64 multiply, ceil, clamp);
  rate attested as its IEEE-754 bit pattern.
- FND-011 resolved: FR-080 replica serve-failure falls back to the shard
  owner; replica copy/attestation reclaim assigned to FR-082 (with the
  known no-deletion-path gap note).
- FND-012 resolved: FR-086 deterministic membership-order-prefix subset.
- FND-013 resolved: FR-080 states frozen-membership tombstone semantics
  identical across fused/unfused/crown paths; FR-086 marks the cached
  flag advisory with owner authority at expansion time.
- FND-014 resolved: FR-088 defines N as snapshot-captured records
  (excludes dead-at-capture; later tombstones do not re-size).
- FND-015 resolved: FR-090 exact seed policy claimable only when crown
  capacity ≥ sample_count; coarser crowns are labeled seed-set changes by
  construction.
