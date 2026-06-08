# Task 87: TurboQuant Candidate Batching

Status: active
Owner: coder. One coder, one branch.
Priority: 1 (TurboQuant scan latency follow-up)

## Why

Task 86 landed the SPIRE TurboQuant no-QJL 4-bit LUT scoring alignment and
left candidate-batching kernel work explicitly out of scope. Current scan paths
still score many TurboQuant candidates through one-payload-at-a-time loops even
when an AM has a contiguous candidate payload batch. RaBitQ already has a
batch-max helper for this shape; TurboQuant should expose the same kind of
quantizer-owned surface before AMs grow more local scorer loops.

## Scope

Add a TurboQuant candidate-batching abstraction that:

- treats TurboQuant no-QJL 4-bit and TurboQuant TQ+ as first-class quant types
  the abstraction must accommodate;
- starts with an exact no-QJL 4-bit LUT batch/max path over contiguous payloads;
- preserves the current per-candidate score semantics and candidate ordering;
- is callable by AM code without exposing AM-specific payload iteration details
  in the shared quantizer layer;
- records packet-local parity evidence before broader scan-path rollout.

TQ+ support is a contract requirement for the abstraction, not a requirement to
promote or reland the TQ+ format in this task. Task 89 owns TQ+ format design,
cross-AM validation, second-corpus evidence, and streaming-insert drift checks.

## Non-Goals

- Do not reland `turboquant_tqplus` or any operator-visible TQ+ storage format.
- Do not change durable on-disk format.
- Do not add a new benchmark sweeper; use `ecaz bench suite` for measurement
  matrices.
- Do not compare TurboQuant against other quantizers except where needed to
  explain why a batch abstraction matches an existing local pattern.

## Implementation Order

1. Land the canonical task definition on the Task 87 branch.
2. Add a shared TurboQuant batch/max helper for no-QJL 4-bit LUT scoring.
3. Route the narrowest existing contiguous candidate path through that helper.
4. Add focused parity coverage for batch scores versus the existing scalar loop.
5. Measure a packet-local scan-latency slice before broadening to other AMs or
   adding lower-level SIMD/block-layout work.

## Exit Criteria

- A canonical `reviews/task-87/` packet records the code slice and evidence.
- The first code slice has parity coverage proving unchanged scoring semantics.
- Any scan-path change is PG18-validated or explicitly justified as static-only.
- No new unsafe blocks.
- No TQ+ operator-visible format or reloption lands in this task.
