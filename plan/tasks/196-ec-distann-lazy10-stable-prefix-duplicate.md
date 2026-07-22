# Task 196: ec_distann Lazy10 Stable-Prefix Duplicate Attribution

Status: **proposed** (2026-07-21). Priority: P1 correctness/efficiency
follow-up from Task 193 packet 005.

## Why

Task 193's post-measurement correctness drill failed before exercising the
prepared-plan candidate. With `owner_payload_plan_cache=off`, production-shape
lazy10 materialization, and the 100k trained-head fixture, the attribution
guard reported:

`EC_INTERNAL: stable-prefix deepening re-requested 1 remote payloads`

Task 191's original correctness packet passed the same invariant, so this must
be reproduced and attributed rather than treated as expected behavior. The
guard is benchmark-feature-only; a normal release may silently perform the
duplicate request.

## Goal

Reproduce the first failing semantic scenario on the current release code,
identify whether duplicate ranked IDs, prefix mutation, window-boundary
selection, or scan-state reuse causes the re-request, and fix the narrow root
cause without changing ordering, visibility, failure, or BW×H semantics.

## Required evidence

1. A checked-in `ecaz bench suite` reproducer using the real 100k physical
   fixture and an explicit scenario label.
2. Before/after counters proving zero duplicate remote payload requests across
   fewer/exact/more-than-window, rejected-prefix, null, external-TOAST,
   mixed-owner, and later-owner-outage cases.
3. If scan/materialization behavior changes, isolated 10k/50k/100k recall,
   latency, and storage A/B evidence before closeout.

## Non-goals

- Task 193 prepared-plan reuse; its measured candidate was not useful.
- Traversal beam/hop changes; Task 194 owns those.
- Relaxing or removing the duplicate-request invariant.
