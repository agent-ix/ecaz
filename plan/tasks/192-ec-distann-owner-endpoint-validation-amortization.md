# Task 192: ec_distann Owner Endpoint Validation Amortization

Status: **in progress — reopened by review** (2026-07-21). Priority: P1 latency follow-up to
Task 187's STOP. Roadmap candidates: `MAT-37` and `MAT-38` (their
"catalog/open share" trigger is now measured — see Entry gate).

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`.

## Why

Task 187's fresh 100k attribution
(`reviews/task-187/001-post-materialization-baseline/`) shows the retained
lazy10 production point at 0.9625 recall / 22.40 ms warm mean, with the
materialization sub-stage counters decomposing the largest stage. Per scan
(50-sample means, 2 concurrent owner requests per scan):

- `materialize_owner_endpoint_work` 15.360 ms =
  `owner_open_validate_work` 6.722 + `owner_payload_sql_work` 8.340 +
  `owner_node_lookup_work` 0.292;
- `materialize_request_roundtrip_work` 17.044 ms, so wire + encode + decode
  is only ~1.68 ms/scan;
- coordinator blocking wait `materialize_request_wait` 9.991 ms ≈ the
  concurrent critical path `owner_endpoint_critical` 8.992 ms.

Roughly 90% of the 10.018 ms `remote_materialize` stage is therefore
owner-side compute that existing counters already attribute. The largest
per-request component is open/validate: ~3.36 ms per owner request re-running
frozen-generation lookup and attested-generation/epoch validation that is
invariant for a hot connection serving one published generation. This is
amortizable session state, not per-row work — the measured causal target
Task 187 lacked for transport, but which materialization already has.

## Goal

Advance exactly one bounded candidate: cache validated per-generation owner
endpoint state (frozen-generation lookup, relation open/validate,
attested-generation checks) on the owner session across requests, keyed by
attested generation and invalidated on epoch change. Target the 6.72 ms/scan
open/validate component; predicted 3–6 ms off the 22.40 ms 100k warm mean if
traversal expansion requests share the same endpoint plumbing, less if only
materialization benefits. Per the standing rule this prediction is not a
finding until the A/B proves it.

## Entry gate

Task 187 closed STOP with packets 001–004 accepted (round-2 feedback
2026-07-21). The trigger recorded for MAT-37/MAT-38 — a measured
catalog/open share — is satisfied by `owner_open_validate_work` 6.722 ms/scan
at 100k on release build `fe98ea2cc`.

## Hard constraints

- **Epoch fencing is preserved exactly** (FR-082, ADR-085): cached state is
  keyed by attested generation; any epoch/generation change observed on the
  connection invalidates the cache before serving; a request against a
  retired epoch must fail identically to today. A stale-epoch/publish-cycle
  test is required, not optional.
- No protocol, wire-format, result-shape, or placement change. Those belong
  to Tasks 190/193/194.
- Recall, ordering, result identity, tombstone/visibility semantics, and
  NFR-019 work bounds unchanged; result identity asserted per arm.
- Bounded cache: per-session, per-generation, with explicit size/lifetime
  caps recorded in the request packet.

## Phases

1. **Pre-registration.** Freeze the baseline (Task 187 packet 001 counters
   qualify if the binary is unchanged; otherwise re-run the same suite
   config). Pre-register the single candidate and its predicted counter
   movement (`owner_open_validate_work` down; `payload_sql`, wire, and
   traversal counters unchanged unless expansion shares the path — state
   which is expected).
2. **Isolated A/B at 100k.** Byte-identical fresh generation, cached vs
   uncached arms in one paired run, stage counters on. A stage-local win
   that does not move end-to-end warm mean/tails is rejected.
3. **Full scale.** Only an end-to-end useful candidate proceeds to the
   standard 10k/50k/100k recall + latency + storage matrix via
   `ecaz bench suite`, plus the epoch-change/failure drill.
4. **Decision.** PROMOTE to a separately reviewed productionization slice or
   STOP with the negative result recorded in the roadmap ledger.

## Decision

STOP. The proposed resolved-schema cache was not advanced: it would bypass
live row-tier catalog validation and no paired A/B artifact completed. The
existing descriptor-generation cache remains unchanged.

## Required review packets

1. `reviews/task-192/001-preregistration-baseline/`;
2. `reviews/task-192/002-isolated-ab-100k/`;
3. `reviews/task-192/003-full-scale-decision/`.

## Non-goals

- Per-row payload SQL cost (`owner_payload_sql_work` 8.340 ms/scan) — Task
  193, sequenced after this task's disposition so attribution stays
  per-change.
- Traversal transport counters and candidates — Task 194.
- Binary RPC, shared-memory transport, replication, placement — Task 190.

## References

- `reviews/task-187/001-post-materialization-baseline/artifacts/` (baseline
  counters) and `reviews/task-187/004-full-scale-decision/feedback/`
  (derivation and recommendation).
- Roadmap `MAT-37`, `MAT-38`; FR-082; ADR-085 D10/D12; NFR-019/NFR-020.
