# Manifest — Task 165 M3 slice 5: multinode scan materialization

- head SHA: this commit (task-165-ec-distann-m3)
- task bucket / packet: reviews/task-165/005-multinode-scan
- lane / fixture: Intel local, ec_distann, real DBpedia 10k (dim 1536), m2_10k_idx
- storage format: index format v2; isolated one-index-per-table (m2_10k_corpus)
- build profile: release (ecaz_build_profile = release, verified)
- command: `ecaz dev sql --db ec_distann_m2 --file reviews/task-165/005-multinode-scan/scan-compare.sql`
- timestamp: 2026-07-08
- key result (artifacts/scan-compare.log): 2-node loopback amgettuple scan
  top-10 == single-node baseline — base_minus_two=0, two_minus_base=0,
  order_identical=t.

## Note

Loopback substrate (ADR-085 D2): both roster nodes point at the same instance,
so the coordinator resolves each remote hit's heap TID from its own full
directory. A real multi-node deployment ships row data from the owning node
(CustomScan materialization follow-up); this slice makes the multi-node read
path return correct user-facing rows through `amgettuple`, unblocking the 50k
multinode recall bench.
