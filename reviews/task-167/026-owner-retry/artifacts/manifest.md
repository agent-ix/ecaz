# Task 167 packet 026 artifacts

- Packet: `reviews/task-167/026-owner-retry`.
- Task: 167; packet remains review-open and is not merge evidence.
- Product checkpoint `79afb0d82` is now installed with
  `--release --no-default-features --features pg18`; no runtime preflight has
  completed yet. Existing diagnostic evidence is from `563cb18f7`. Harness
  checkpoints: `88bc8a57d`, `48ca7caea`, and
  `7e11f8322`, and `ac90e38a7`.
- Installed build command:
  `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --no-default-features`.
- Installed binary provenance: SHA
  `79afb0d826ce5f382945c5ed891e3411b30aa1ba`; feature set is
  `--no-default-features --features pg18`. Runtime preflight remains pending.
- The current-head attempt is captured under
  `artifacts/production-current-79-10k-fresh/`: fixture initialization fails
  before producing `results.jsonl` because the external cluster root is
  mounted read-only. This is a blocker record, not benchmark evidence.
- Matrix runner/config: `ecaz bench suite` with
  `artifacts/task167-owner-retry-suite.json`; graph degree 5, three physical
  owners, PG18, production feature set. The bespoke config is intentional: it
  isolates Task 167 owner-retry/physical-DML behavior and uses the task's
  physical 10k/50k/100k corpus lanes rather than the mutable current-lane
  index.
- Cluster policy: all intended run directories are under
  `/home/peter/.ecaz/clusters/`, outside the repository and Cargo target.

## Results that are usable

The existing 10k and synthetic runs are diagnostic results from installed
extension SHA `563cb18f7`, not current-head closeout results:

- Production preflight reports `features=pg18`, release profile, and no debug
  override.
- Natural retry attribution is positive with the forced probe disabled:
  10k `churn_retries=37`, `steady_retries=0`; synthetic degree-8
  `churn_retries=99`, `steady_retries=0`. Attribution is owner-local and is
  read from the side relation, not the 2PC intent table.
- The old 10k physical benchmark used ten measured iterations. The new
  50k/100k configuration requests five measured iterations plus two warmups;
  those results are not yet complete.
- 10k inserted-neighborhood fresh-rebuild recall is `0.133333` in the spare-slot
  run. This is a failure signal, not a passing acceptance result.
- The same run reports `initial_neighbors=5`, `before_neighbors=3`, and
  `final_neighbors=3`; the exact-degree saturation gate correctly fails.
  The controlled target check also misses one writer id. The synthetic
  degree-8 case keeps degree 8 but misses both controlled writer ids.

## Incomplete or rejected runs

- The old 50k/100k matrix was not decision-grade: one attempt used an
  unsuitable head cap, another was interrupted before suite completion, and
  the latest indexed rerun failed before fixture setup because the host root
  filesystem was read-only. These logs are retained for diagnosis only.
- The append A/B values in the old run were small-sample diagnostics. The
  production-path GUC was corrected at `3c162f69d`, and the harness now uses
  five 32-row trials with a pass condition requiring no throughput regression
  and no increase in backlink amendments. The corrected extension was not
  installed, so no current-head A/B result exists yet.
- No packet line claims 10k/50k/100k closeout, no large-scale pass is inferred
  from a partial log, and no forced retry probe is used as proof of the natural
  race.

See the packet-local raw logs and `cited-results-final.log` for exact result
lines. The packet must remain open pending a clean current-head production
install and rerun.
