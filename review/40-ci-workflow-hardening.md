# Review Request: CI Workflow Hardening

Scope:
- `.github/workflows/ci.yml`

What changed:
- Added workflow-level concurrency cancellation so stale CI runs on the same ref do not pile up.
- Set `PGRX_HOME` explicitly for the pg17 job to match the project’s validation convention and keep pgrx state in a stable temporary location.
- Left the existing command set unchanged: fmt, clippy, unit tests, unsafe audit, pg17 integration tests, and license audit are still the active gates.

Review focus:
- Whether the concurrency policy is reasonable for this repository’s CI usage
- Whether setting `PGRX_HOME` at the job level is the right scope for the pg17 workflow
- Whether the workflow still reflects the intended CI gates without adding noise or unnecessary maintenance burden

Questions to answer:
- Is the `ci-${{ github.workflow }}-${{ github.ref }}` concurrency group safe for push and PR runs?
- Should `PGRX_HOME` be workflow-wide or limited to the `pgrx` job as implemented?
- Does this keep the workflow minimal while still making the pgrx job more reproducible?
