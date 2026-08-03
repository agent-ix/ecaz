# Task 213 implementation artifacts

- Task bucket: `reviews/task-213/`; packet: `002-fused-head-hop-implementation`
- Code head: `a8b1699528e593b45f55fc25329199714d4627ff`
- Installed PG18 release extension: `a8b1699528e593b45f55fc25329199714d4627ff`
  (release profile, committed tree; three-node preflight was unanimous).
- Validation: PG18 focused physical handoff test passed after lifecycle
  coverage was added; see the Task 212 packet's `validation-followup.log`.
- Suite config: `task213-fused-suite.json`.
- Final suite source of truth: `bench-run-final2/suite-manifest.json` and
  `bench-run-final2/results.jsonl` (all 6 steps succeeded).
- Post-fix suite source of truth: `bench-run-postfix/suite-manifest.json` and
  `bench-run-postfix/results.jsonl` (all 6 steps succeeded with the corrected
  plain-arm gate and first-round accounting).
- Command: `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-213/002-fused-head-hop-implementation/artifacts/task213-fused-suite.json --artifact-dir reviews/task-213/002-fused-head-hop-implementation/artifacts/bench-run-final2 --log-file reviews/task-213/002-fused-head-hop-implementation/artifacts/bench-run-final2/suite.log --continue-on-error`
- Corpus query SHA: 10k `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`,
  50k `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3`,
  100k `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.
- Physical isolated one-index-per-table A/B results (recall / recall-run
  mean ms / storage amplification; unfused, fused):
  - 10k `0.9940 / 40.32 / 1.235867`, `0.9985 / 34.91 / 1.235467`.
  - 50k `0.9595 / 53.91 / 1.332667`, `0.9585 / 41.57 / 1.332667`.
  - 100k `0.9145 / 54.73 / 1.351160`, `0.9300 / 40.81 / 1.351160`.
- The latency-run means—the direct mechanism comparison—were unfused versus
  fused: 10k `38.0 -> 32.7 ms`, 50k `53.1 -> 39.8 ms`, and 100k
  `55.8 -> 40.3 ms`. Hop rounds fell `595 -> 470`, `805 -> 632`, and
  `784 -> 596`, respectively. Both arms had crown enabled; the fused first
  round consumed crown-ranked seeds and now reports its requested-id count.
- Post-fix latency means were unfused versus fused: 10k `39.80 -> 33.40 ms`,
  50k `50.30 -> 41.30 ms`, and 100k `51.60 -> 38.90 ms`; post-fix recall was
  `0.9940 -> 0.9985`, `0.9595 -> 0.9585`, and `0.9145 -> 0.9300`.
- The shared release capacity matrix covers fused capacities 512, 2048, and
  4096 at 10k/50k/100k. Capacity 2048 is selected for the opt-in fused
  configuration: it is the latency winner at all three scales and is within
  0.001 recall of capacity 4096 at 100k. Every capacity arm is labeled
  `seed_set_change=true`.
- Post-fix unfused arms reported `crown_seeds_served=0`,
  `fused_head_hops=0`, and `fused_first_round_requested_ids=0`. Fused recall
  arms reported `crown_seeds_served=6400`, `fused_head_hops=200`, and
  `fused_first_round_requested_ids=6400`; fused latency arms reported 1600,
  50, and 1600 respectively.
- Fused provenance is explicitly `seed_set_change=true` at every scale; it
  is not treated as an identity-preserving control. Fused recall counters
  served 6400 crown seeds and recorded 200 fused hops on each recall arm;
  latency counters served 1600 crown seeds and recorded 50 fused hops. Both
  variants recorded zero crown fallbacks.
- The physical retry path now classifies the typed epoch-mismatch error and
  reopens the active generation after resetting stale physical state; other
  search errors remain internal failures.
- Storage rows itemize `ec_distann_crown_cache` as bounded codes-only storage
  (`resident_bytes=resident_bytes_bound=434176` at the 2048-entry arms) with
  coordinator unsharded bytes zero.
- Artifacts retained in this packet are only the suite manifest, structured
  results, and compact per-arm summary logs. Corpus data, predictions,
  operational logs, and PostgreSQL clusters are not committed/resident.
