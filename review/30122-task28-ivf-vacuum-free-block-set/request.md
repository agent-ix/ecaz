# Task 28 IVF Vacuum Free-Block Set Trial

## Scope

This packet records a rejected F2/A3 experiment: replacing the single backend-local posting free-block hint with a per-list set populated by vacuum.

The experiment was not kept in code because it regressed the nlists=64 churn smoke.

## Result

At head `4323499a` plus an uncommitted free-block-set patch:

- nlists=32 still converged at `4,464,640` bytes through cycle3.
- nlists=64 grew from `4,472,832` bytes at build to `5,062,656` bytes at cycle3.
- nlists=64 cycle3 refill was `162,264.328 ms`.

That is worse than the pushed single-block hint packet 30121, where nlists=64 cycle3 was `4,997,120` bytes and `135,267.117 ms`.

## Decision

Do not land the free-block-set variant. The extra candidate set did not solve physical convergence and added enough per-insert overhead to make the pathological nlists=64 case slower.

The next A3/F2 slice should target structure rather than more backend-local candidate tracking:

- explain why nlists=64 still needs new pages under same-shape delete/refill churn;
- inspect per-list range boundaries and mixed-list page behavior;
- preserve the current single-block hint as the best measured local improvement until a stronger reuse design is implemented.

## Artifacts

- `artifacts/ivf_sustained_churn_hintset_smoke.log`
- `artifacts/manifest.md`
