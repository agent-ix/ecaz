# Review request — Task 211 P0: head scaling law spec

- Task: `plan/tasks/211-ec-distann-head-scaling-law.md`, phase P0 (spec-first)
- Packet: `reviews/task-211/001-head-scaling-law-spec/`
- Spec artifact: `spec/functional/distann/read/FR-088-distann-head-scaling-law.md`
  (commit `fe9bfabbc`, branch `task-203-ec-distann-conformance`)
- Date: 2026-08-01. Coder: fable (Claude Fable 5)

## What to review

FR-088 expresses head sizing as `C = clamp(ceil(rate × N), floor, ceiling)`
resolved at T2 from the build's captured record count, manifest-attested and
digest-bound, with the pre-existing explicit `head_index_cap` retained as the
fixture/pin override (precedence: explicit cap wins; rate = 0 disables the
law). Shipped default stays the explicit cap until the P2 sweep lands a rate.

Landed against the Task 214 elevated structure
(`spec/functional/distann/read/`), on the rewritten FR-080 (sharded
membership-only head — coordinator cost of head growth is O(C) ids only).

Design points worth challenge:

1. **Reloption shape** — `head_sampling_rate` + `head_cap_floor` +
   `head_cap_ceiling` as three reloptions, with floor defaulting to the
   current 4096 and ceiling to the frozen v1 domain max. Alternative: a
   single packed option. The three-knob form keeps the law readable and each
   bound independently pinnable.
2. **Trained-head reconciliation** — the trained policy requires C = 4096
   exactly (shipped validity rule); FR-088 makes an incompatible law/policy
   combination fail the build rather than silently pin. Is fail-closed right
   here, or should trained generations implicitly pin the explicit cap?
3. **AC-5's hop-count honesty rule** — an arm improving latency without
   moving hop counts is flagged, not celebrated (hop reduction is the
   mechanism). Counters availability note: `traversal_hop_rounds` is
   currently feature-gated (inventory B3); the implementation phase must
   resolve counter availability for production-build A/Bs.
4. **Scale bound** — sweeps stop at 100k (user ruling 2026-08-01); 1M+
   re-validation is an explicit later gate. Specced under "Verification
   scope", including the honest no-winner outcome.

## Validation

`quire validate` clean (advisory EARS warnings only). Implementation (P1
build-side law, P2 sweep) is explicitly NOT started — speccing round only.

## Status

Open — awaiting reviewer feedback.

## Update (2026-08-01, same session)

The spec was subsequently hardened by the failure-domain, integrity, and
scope-boundary analyses (`spec/reviews/{failure-domain,integrity,scope-boundary}.md`,
findings + resolutions recorded there). Material additions: see the
resolution lists for the FR this packet reviews.
