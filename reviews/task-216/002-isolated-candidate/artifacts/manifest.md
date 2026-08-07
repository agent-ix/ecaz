# Task 216 packet 002 manifest

- Packet: `reviews/task-216/002-isolated-candidate/`
- Status: measured; candidate rejected at isolated 100k gate
- Candidate: MAT-15 only — packed null bitmap + cumulative `int8` offsets + flat `bytea`
- Control: current per-column `bytea[]` payload representation
- Required first scale: fresh physical 100k A/B
- Runner: `ecaz bench suite` with `artifacts/task216-mat15-100k.json`
- Topology: normal PG18 release, coordinator plus three sharded owners; one
  index per owner table, no coordinator full-graph replica
- Parameters: BW4/H100/L32, graph degree 32, build shards 1, head cap 4096,
  persisted-head seeds 32, hop rounds 100, top-k 10, 50 iterations / 10
  warmups, `ec_real_100k`
- Isolation: control is run from the pre-change release commit; candidate is
  run from the MAT-15 commit. Each arm provisions a fresh generation; the
  resulting seed digest/index drift is a known lane defect and prevents
  ordered-identity claims. No MAT-21, Task 215 BW64/H8 defaults, or other
  traversal/materialization changes are included.
- Artifact rule: suite manifests, `results.jsonl`, cited summaries, and
  validation logs remain under this packet's `artifacts/`; cluster run
  directories remain outside the repository and are removed after capture.
- Fault drills: explicitly skipped in the checked-in config
  (`skip_fault_drills: true`) at the diagnostic gate; no outage/failure-drill
  coverage is claimed.

- Control head SHA: `e8f15ab0c68887c176a260107fe826c402c2f827`
- Candidate head SHA: `6662b302f8370695320dcb36edda3cd291c8c1bc`
- Control run: `2026-08-07T00:45:02Z` to `2026-08-07T01:19:23Z`; command:
  `ecaz bench suite run --config reviews/task-216/002-isolated-candidate/artifacts/task216-mat15-100k.json --artifact-dir reviews/task-216/002-isolated-candidate/artifacts/control`
- Candidate run: `2026-08-07T01:59:35Z` to `2026-08-07T02:31:36Z`; command:
  `ecaz bench suite run --config reviews/task-216/002-isolated-candidate/artifacts/task216-mat15-100k.json --artifact-dir reviews/task-216/002-isolated-candidate/artifacts/candidate`
- Both suite manifests report `status=succeeded`; the CLI additionally reports
  NFR-021 actual `unavailable` because this diagnostic run is below the
  registered conformance scale. Topology, remote-owner, and storage gates
  passed in both runs.
- Artifact SHA256: control `results.jsonl`
  `0833af02189bf11a38b99cbf2e53748bc043ee7f5e7194a5602d549359a32352`,
  `suite-manifest.json`
  `758b51b7017afed74daa285441a7158e5a2806fd5782cbd345e81bcafa135190`,
  summary `bab64da602a84300cbef93d17135802d2d827a0edf7d6bd21dace4c3c0488a8d`;
  candidate `results.jsonl`
  `ac97d9b15cc0626988189cdf01f962323c0d5426eb1f7ed61376c5097f187482`,
  `suite-manifest.json`
  `f230fcf1a52000c6a44a395e37fb4f3346a74a77914e97590d112913f3b0fe15`,
  summary `1731315be1f774a9f5fc6ec138fee68f32e1de1978a1c791a9c9fa4213ff05d4`.
- Cited physical 100k results: control recall `0.9275`, mean/p50/p95/p99
  latency `40.60/39.30/54.30/57.20 ms`, storage
  `2496659456 B` and amplification `1.351173`; candidate recall `0.9295`,
  mean/p50/p95/p99 latency `86.10/85.00/113.70/127.00 ms`, storage
  `2496634880 B` and amplification `1.351160`.
- Stage attribution: `custom_scan_total` was `38.34` versus `83.78 ms`,
  `materialize_owner_payload_sql_work` was `40.376` versus `118.422 ms`
  owner-summed, `materialize_coordinator_decode` was `0.076` versus `0.096
  ms`, and payload bytes were `576576` versus `576945`. The owner SQL stage
  is the regression site; the coordinator decode ceiling is `0.076 / 40.60 =
  0.19%` of the control scan.
- The candidate's physical prediction artifact differs from control in 2 of
  200 ordered query rows. The single-surface prediction artifact is byte
  identical. The physical seed digests also differ (`d25e8d4d...` versus
  `0e623a8d...`) because each arm rebuilt a fresh generation. This is a lane
  defect, not an unexplained or MAT-15-attributed regression.
- Install provenance logs are packet-local:
  `candidate-install.log` SHA256
  `2816cc472e28df72769ab08f500ee3c59988396ee0a847dd10a785dd9d326306`,
  `candidate-install-rerun.log` SHA256
  `2816cc472e28df72769ab08f500ee3c59988396ee0a847dd10a785dd9d326306`, and
  `candidate-install-isolated.log` SHA256
  `05343f9ff268ba655b1b8634717c13f6267151f609e7726122a3b985c54047fa`.
- Decision: **STOP**. MAT-15's measured addressable ceiling is 0.19% on this
  profile and response bytes are unchanged; the slower SQL implementation is
  secondary evidence. Packet 003 full-scale 10k/50k/100k measurement is not
  authorized because the candidate was not useful at the required first gate.
