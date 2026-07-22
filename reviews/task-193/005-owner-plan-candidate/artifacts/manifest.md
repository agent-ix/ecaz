# Artifact manifest — Task 193 packet 005

- Task bucket / packet: `reviews/task-193/005-owner-plan-candidate/`.
- Implementation SHA: `e444f6474`.
- Lane: local Intel, three isolated PG18 owner instances.
- Fixture: `ecaz bench suite`; one physical generation shared by both A/B
  arms plus a separate same-data single-index control.
- Storage / rerank / search: trained exact landmark head, RaBitQ stored
  neighbor codes, exact co-located row-tier rerank, lazy10, BW=4/H=100.
- Isolation: both arms explicitly disable the Task 192 validation cache. The
  control explicitly disables and the candidate explicitly enables only the
  Task 193 owner payload plan cache.
- Protocol: 200 recall queries / 2,000 trials and 10 warmups + 50 measured
  latency iterations. Stage counters are enabled. Promotion correctness drills
  are omitted from the decision rerun after the cache-off baseline exposed a
  pre-existing stable-prefix duplicate-request failure; the candidate already
  failed its performance gate and cannot advance to promotion.
- Corpus/query: `ec_real_100k`; corpus TSVs are intentionally not committed.
- Suite config: `task193-owner-plan-100k.json`.
- Suite audit: passed, one step.
- Validation: strict PG18 attribution-feature clippy passed with warnings
  denied; the focused CLI variant tests passed; the runner build completed.
- Installed extension preflight: release target and installed PG18 library are
  both 24,244,984 bytes with SHA-256
  `ec58009be20adf9db45af01fcc9bf0a947b9ec893ee6541f9c47d194f5ea8031`.
- Planned command: `target/debug/ecaz bench suite run --config
  reviews/task-193/005-owner-plan-candidate/artifacts/task193-owner-plan-100k.json
  --database tqvector_bench --log-file
  reviews/task-193/005-owner-plan-candidate/artifacts/suite-run.log`.
- Decision command: `target/debug/ecaz bench suite run --config
  reviews/task-193/005-owner-plan-candidate/artifacts/task193-owner-plan-100k.json
  --database tqvector_bench --log-file
  reviews/task-193/005-owner-plan-candidate/artifacts/suite-run-decision.log`.
- Runner SHA: `e444f6474`; extension SHA
  `fb0c512bf3bb9c7358ea905bf4e8565bd53fc181`, unanimous release profile.
- Decision suite: 2,084,789 ms; 1 succeeded, 0 failed, 0 missing, 0 stale.

Node logs, fixture transcripts, single-control raw logs, the failed drill
attempt, and generated corpus/truth data are operational exhaust and are not
committed.

## Pre-run files

- `release-install.log`: release build/install transcript.
- `suite-audit.log`: checked-in suite shape/input audit.
- `validation.md`: exact validation commands and release identity.

## Decision artifacts

| Artifact | SHA-256 | Purpose |
|---|---|---|
| `task193-owner-plan-100k.json` | `b329f0881ee41dc8fc6aa960baded340c413e133f81cb93c1b9a62104022d137` | Isolated A/B config |
| `decision-run/suite-manifest.json` | `387a79a39bfebcbd26327e2a241610c7770df995fd90225325200b770f0a5468` | Success state/provenance |
| `decision-run/results.jsonl` | `9055168a8d833cfca1554ec9ff246dd574ae8a0c8c67c0885a67ada3bd55fa12` | Structured result rows |
| `decision-run/owner-payload-plan-ab-100k/distann-multinode-summary.log` | `59ed08600061f38281fdb4914c5245243e30332499c2389d4a5194dfb1402e90` | Parsed summary |
| `decision-run/owner-payload-plan-ab-100k/physical-owner-plan-uncached-recall.log` | `8396786b4fda7e8d2e6572a6963c36476d482c47cbd33d57a39d3ae3e4c22bee` | Control recall |
| `decision-run/owner-payload-plan-ab-100k/physical-owner-plan-uncached-latency.log` | `4c78051189e2c594a407b04794a2206da943baa0db0cd1ffad684c61ab70c46a` | Control latency/counters |
| `decision-run/owner-payload-plan-ab-100k/physical-owner-plan-cached-recall.log` | `3c3583c6b8907db61a33e5a6a23587927681133474943d2eb1637c9ed07d8259` | Candidate recall |
| `decision-run/owner-payload-plan-ab-100k/physical-owner-plan-cached-latency.log` | `c06127e0b010bd5f20817467b1aedab54b51906e25d4555e5db112e4ebc6b0af` | Candidate latency/counters |

## Key results

- Recall uncached/cached: `0.9625/0.9625`.
- Warm mean `23.60/23.50 ms`; p50 `23.50/23.20`; p95 `26.80/26.40`.
- Owner payload SQL `8.746651/8.599735 ms/scan`; owner open/validate
  `6.906081/6.854807`; remote materialize `10.432690/10.324276`.
- Physical generation storage: `2,496,659,456` bytes both arms.
- Decision: STOP; only 0.147 ms (1.7%) moved in the intended stage and
  end-to-end mean moved 0.1 ms.
