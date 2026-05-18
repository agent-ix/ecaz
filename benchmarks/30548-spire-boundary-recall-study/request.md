# Review Request: SPIRE Boundary Recall Study

- Measurement head: `54eece753ac9e262a88a3dca894dd6f44b6d897c`
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 5 boundary replication
- Agent: coder1

## Summary

This packet records the first local real-10k recall/storage study for SPIRE
boundary replication.

The compared indexes use the same corpus and index shape:

- corpus: real 10k fixture from `target/real-corpus/ec_hnsw_real_10k`
- rows / queries / dimensions: 10,000 / 200 / 1536
- access method: `ec_spire`
- storage format: `turboquant`
- reloptions: `nlists=32`, `nprobe=24`, `rerank_width=25`
- lanes: `boundary_replica_count=0` and `boundary_replica_count=1`

Raw packet-local artifacts are listed in `artifacts/manifest.md`.

## Results

`boundary_replica_count=1` doubles physical assignment rows and nearly doubles
the SPIRE index bytes:

| lane | base rows | primary rows | boundary rows | index size | index bytes/row |
| --- | ---: | ---: | ---: | ---: | ---: |
| boundary off | 10,000 | 10,000 | 0 | 8.2 MiB | 857.7 B |
| boundary replica 1 | 20,000 | 10,000 | 10,000 | 16.0 MiB | 1673.6 B |

Recall changes are measurable but small on this already-strong fixture:

| nprobe | off recall@10 | rep1 recall@10 | delta | off mean q-time | rep1 mean q-time |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 4 | 0.9950 | 0.9975 | +0.0025 | 40.17 ms | 74.65 ms |
| 8 | 0.9985 | 0.9990 | +0.0005 | 62.59 ms | 120.52 ms |
| 16 | 1.0000 | 1.0000 | 0.0000 | 103.27 ms | 206.87 ms |
| 24 | 1.0000 | 1.0000 | 0.0000 | 139.61 ms | 289.65 ms |

The current local conclusion is that boundary replication is functioning and
produces the expected physical storage fanout, but it is not a clear win for
this real-10k operating point: the low-probe recall lift is marginal relative
to the doubled candidate surface.

## Review Focus

1. Confirm the packet is enough to close the Phase 5 local recall/storage
   evidence item for boundary replication.
2. Check whether the result should keep `boundary_replica_count` default-off
   for now, given the small recall lift on this fixture.
3. Confirm the assignment and storage counters are framed correctly as physical
   overhead, separate from logical deduped scan output.
4. Review the note in the manifest about declaring the new diagnostic SQL
   function in the scratch database before capturing leaf snapshot artifacts.

## Validation

- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18`
- `cargo build -p ecaz-cli`
- `target/debug/ecaz ... corpus load ... boundary_replica_count=0`
- `target/debug/ecaz ... corpus load ... boundary_replica_count=1`
- `target/debug/ecaz ... bench recall ... --sweep 4,8,16,24`
- `target/debug/ecaz ... bench storage ...`
- `psql ... ec_spire_index_leaf_snapshot(...)`

PG17 was not run; this is a PG18 local measurement packet.
