# Task 204 arm-fidelity packet

- Head SHA for the implementation and benchmark source: `239bf0daa` (the installed extension reports its unchanged core SHA `6be6483cb8e890ec96ce1f2cb3670e72e356dcb4`).
- Task bucket: `reviews/task-204/001-arm-fidelity/`.
- Fixture/lane: PG18 `distann-local-multinode`, two physical arms at `ec_real_100k`; owner-control versus coordinator-replica.
- Storage format: rabitq neighbor codes; rerank mode is the default co-located exact-distance path.
- Suite config: `artifacts/task204-two-arm-100k-suite.json`.
- Intended command: `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-204/001-arm-fidelity/artifacts/task204-two-arm-100k-suite.json --artifact-dir reviews/task-204/001-arm-fidelity/artifacts/run-final`.
- Run directory: `/home/peter/.ecaz/clusters/task204-arm-fidelity-100k` (outside the repository and `target/`).
- Isolation: the suite uses one physical three-node fixture with two seed arms; storage rows are emitted inside the arm loop. The fixture is not shared with another packet.
- Benchmark timestamp: 2026-07-29 America/Los_Angeles; suite duration was
  3,078,652 ms and exit code was 0.

## Evidence status

The code checkpoint emits `physical_benchmark_storage`,
`physical_benchmark_storage_ratio`, `physical_benchmark_storage_node`,
`physical_benchmark_storage_relation`, and per-arm
`physical_benchmark_traversal_replica_cache` rows. The focused PG18 test passed;
see `pg18-focused.log`.

The required 100k two-arm fixture completed against the staged real corpus.
The topology rows prove 100,000 source rows distributed as 33,195 / 33,432 /
33,373 across three owners, with zero non-owned rows and zero orphans. The
owner-control arm reports `cluster_graph_side_bytes=830144512` and
`cluster_index_space_amplification=1.351147`; the coordinator-replica arm
reports `cluster_graph_side_bytes=2489663488` and
`cluster_index_space_amplification=4.052187`. The replica relation row
reports 100,000 copied records, 1,315,200,000 copied bytes, and 1,659,518,976
relation bytes. See `run-final/results.jsonl` for structured rows and
`run-final/storage-two-arm-100k/distann-multinode-summary.log` for the cited lines.

The command used `--skip-recall` and `--skip-single-control` because this is a
storage-fidelity packet; the two physical arms were both measured at BW=4 and
H=100 with five warm iterations after two warmups. The release preflight
reported the source SHA with a `-dirty` suffix because packet artifacts were
being written in the worktree; the implementation source was unchanged from
the stated head SHA. The exact external cluster was removed after evidence
capture.

The corrected reread of the committed Task 198/199 artifacts is in
`corrected-198-199-reread.md`.

## Artifact inventory

| artifact | SHA-256 | purpose |
|---|---|---|
| `run-final/results.jsonl` | `7b8fefcd93748fa7b51308b6354fb53da2297654cf1672954c04c56171fba989` | structured suite rows cited by the request |
| `run-final/suite-manifest.json` | `38c27e950952f60429c8a9a019cec63ea1b2fa84abae1f4a657f772dde7b1d7b` | command, config, duration, and exit status |
| `run-final/storage-two-arm-100k/distann-multinode-summary.log` | `84a5fc219a0ef48fc8548de8f5387897d04b966e8b54b43d90a9fa7ce674b42a` | topology, arm storage, ratios, and replica rows |
| `run-final/storage-two-arm-100k/physical-owner-control-latency.log` | `1767690c005fbfaa42c40b37b258c146ab7d190a525e9bfc5f029424d8eb9177` | owner-control five-trial latency table |
| `run-final/storage-two-arm-100k/physical-coordinator-replica-latency.log` | `3f04119554ff773d00dff1b5348e67c07b235806c74efe95cce645b0b9bfd869` | coordinator-replica five-trial latency table |

All artifacts were produced by `ecaz bench suite`; no corpus TSV or cluster
data is committed. The exact external cluster was removed after the summary
and structured results were captured.
