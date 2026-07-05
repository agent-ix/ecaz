# Task 49: Hardening CI Governance and Lane Retargeting

Status: **implemented** — addresses the meta-gaps Task 34 left:
synthetic-harness lanes that report "passing" without exercising ECAZ code, a
weakened `make test` that ships to CI, and the absence of a documented
promotion path from local to PR-blocking to nightly.

Task 49 landed before the structural hardening tasks. It repaired the trust
ledger from Task 34 first; Task 41 can now remove PG-resource unsafe sites by
construction, with Tasks 36/37, 39, 40, and 43 following per environment
readiness.

## Scope

Three governance areas:

1. **Lane retargeting.** Fix the four Task 34 lanes whose harnesses do
   not exercise ECAZ code:
   - `hardening/rudra/` — synthetic ItemPointer reimplementation.
   - `hardening/flux/` — three fake arithmetic functions.
   - `hardening/loom/` — generic AtomicUsize CAS, no ECAZ types.
   - `hardening/shuttle/` — inline `Candidate` struct, no ECAZ types.
2. **`make test` restoration.** Revert or rescope commit `773c75b4`
   ("Route make test through local unit lanes") so `make test` runs
   the actual extension test suite on CI (Linux) while keeping a
   `test-hardening-local` lane for the macOS pgrx loader workaround.
3. **Promotion policy.** Document and enforce the local → PR-blocking
   → nightly → weekly promotion ladder for every hardening lane,
   including the explicit signal each lane must produce before
   promotion.

## Why

Task 34's review surfaced three concrete governance failures:

- Four lanes "passing" with zero ECAZ coverage. Reviewers reading
  `request.md` reasonably interpret "make rudra ran SendSyncVariance
  analysis" as evidence about ECAZ. It is not.
- `make test` was silently weakened: it used to run `cargo test`
  (which works on Linux even when broken on macOS); after commit
  `773c75b4` it runs CLI-only tests plus a tiny standalone harness.
  CI is `ci-quick: fmt-check lint test layout-check audit-unsafe`, so
  the regression flows to CI — the main extension test suite is no
  longer gated.
- There is no rule for when a lane gets promoted from local to PR or
  to nightly. Task 34 documents an aspiration ("promote after
  burn-in") but no criteria.

Without governance, hardening lanes accumulate as decorative tooling
rather than enforcement, which is the worst of both worlds: the cost
of running them without the benefit of trusting them.

## Approach

### Lane retargeting

1. **Rudra**: Rudra's pinned 2021 Cargo cannot resolve the workspace
   today (`hashbrown ^0.15`). Two acceptable resolutions:
   - Wait for an upstream Rudra release that supports newer Cargo, then
     point it at the real workspace.
   - Document Rudra as a *manual one-shot audit* tool (not a recurring
     lane) and remove `hardening/rudra/` and the `make rudra` lane.
     File any current findings as follow-ups.
   Either resolution is fine; the current synthetic harness is not.
2. **Flux**: Replace `hardening/flux/src/lib.rs` with real Flux
   annotations on `ProdQuantizer::new` and `Page::tuple_at` (Task 44
   covers the substance). If Flux annotations are deferred to Task 44,
   remove the synthetic lane in the interim — do not leave a passing
   lane that proves nothing.
3. **Loom**: Replace synthetic CAS with a Loom test importing the real
   parallel-build worker-slot atomic (Task 40 covers the lift). If
   Task 40 is not yet landed, remove `hardening/loom/` so the build
   surface is honest.
4. **Shuttle**: Same pattern. Either import real SPIRE coordinator
   types via Task 40's lift, or remove until then.

### `make test` restoration

5. **Revert the routing change.** `make test` runs `cargo test`. The
   pgrx loader issue on macOS is a separate concern — document the
   workaround as `make test-local` (CLI + careful harness) and have
   developers on macOS use that. CI on Linux uses `make test` and
   exercises the full suite.
6. **Update CI alias.** `ci-quick` uses `make test`. Developers on
   macOS can opt in to `test-local` for fast inner loop; this is a
   documentation change, not a Makefile change.

### Promotion policy

7. **Tiers.**
   - **Local**: any new lane lands here first. No signal required
     beyond "the tool runs and produces output."
   - **PR-blocking**: requires (a) ≥ 2 weeks of nightly passing
     locally, (b) measured PR-time wall clock < 5 min, (c) at least
     one captured signal — either a finding or an injected-bug
     validation.
   - **Nightly**: lanes too slow for PR but that produce useful
     signal. Failures page or file follow-ups but do not block.
   - **Weekly / manual**: long campaigns (mutation testing, AFL,
     soak) that produce signal only over hours.
8. **Promotion checklist.** Every promotion lands as a packet with:
   - lane name and current tier,
   - target tier and rationale,
   - measured wall-clock budget,
   - capture of at least one finding (or an injected-bug validation
     run),
   - rollback plan if the lane becomes flaky.
9. **Demotion policy.** A lane that fails for unrelated reasons more
   than N times in a window gets demoted automatically with a packet
   filed for repair. Prevents lane rot blocking unrelated work.

### Lanes / documentation

10. **Make lanes:**
    - `make hardening-validate` — runs a CI-self-test that verifies
      each lane in `hardening-local` actually exercises ECAZ code
      (greps for ECAZ symbol references in the lane's compiled
      binary; fails on synthetic-only).
    - `make hardening-tiers-report` — prints every lane and its
      current tier, time budget, last-passing SHA, and any
      outstanding promotion or demotion proposal.
11. **`docs/hardening-governance.md`** — authoritative source for the
    tier model, promotion checklist, demotion policy, and current
    inventory.

## Validation

- `make hardening-validate` flags the current four synthetic lanes and
  any future synthetic lanes immediately.
- `make test` runs `cargo test` and exercises the full extension
  suite on Linux CI.
- Every lane in `hardening-local` and `hardening-nightly-local` is
  listed in `docs/hardening-governance.md` with its tier.
- A no-op PR shows no demotions; an intentionally broken lane gets
  demoted by the next nightly run.

## Exit Criteria

- No synthetic-only harnesses in `hardening/`.
- `make test` is fully restored on CI.
- Every hardening lane has a documented tier and last-passing SHA.
- `make hardening-validate` blocks PRs that add a synthetic lane.

## Dependencies

- Tasks 40 and 44 produce the lifts that let Loom / Shuttle / Flux
  point at real code; this task either coordinates with them or
  documents the interim deletion.
- Independent of Tasks 36–39, 41–43, 45–48 mechanically, but it gates
  any of their lanes that want to be promoted.
- Should land soon — every week the synthetic lanes remain compounds
  the trust deficit.
