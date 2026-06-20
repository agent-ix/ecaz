# Task 111h Final Closeout Audit

Head SHA: `537c16ca82ed7e7808d19bff91151b0acb6e6465`

Task file: `plan/tasks/111h-ivf-persisted-rerank-format-sweep.md`

This audit closes Task 111h against the current task text and packet evidence.
It adds no new benchmark data. Every performance claim below points to a
packet-local artifact already committed under `reviews/task-111h/`.

## Final Decisions

| Placement / format | Decision | Evidence | Rationale |
| --- | --- | --- | --- |
| `source/f32` | Promote as default/reference | Packets `026`, `027`, `024`, `028`, `029`, `040` | Exact source-vector rerank adds no compact payload storage, is the matched-recall warm-cache reference at 50k/100k/1M, and remained strongest in the 50k cold diagnostic. |
| `table/*` | Abandon as a 111h product path; keep reserved | Packet `034` | Current code rejects `rerank_placement='table'`; a real PostgreSQL-owned compact payload would need new DDL/MVCC/maintenance/storage design. The 111h replacement is `source/f32` for exact table/source storage and `index/*` for persisted compact storage. |
| `index/f16` | Iterate only; do not promote current layout | Packets `024`, `026`, `027`, `028`, `029`, `040` | Recall-neutral versus `source/f32`, but current index storage is much larger: 166.7 MiB vs 13.8 MiB at 50k matched 0.99, 330.1 MiB vs 24.6 MiB at 100k matched 0.99, and 3.1 GiB vs 226.8 MiB at 1M matched 0.99. The 50k cold diagnostic showed no cold latency advantage. |
| `index/rabitq4` | Abandon current 111h candidate | Packets `024`, `026`, `027`, `028`, `029`, `040` | Does not reach 0.95 recall at 50k or larger in the post-v7 warm matrix, and also misses 0.95 in the 50k cold diagnostic at nprobe 200. Any future RaBitQ4 work needs a new fidelity/storage hypothesis. |
| `index/rabitq8` | Iterate only; do not promote current layout | Packets `024`, `028`, `029`, `036`, `037`, `040` | Default clip=2 evidence was too pessimistic; packet `036` proved clip=4 can reach about 0.992 recall at 100k nprobe 200, and packet `040` reached 0.9930 at 50k nprobe 200. It still has a larger IVF index than `source/f32` and did not beat `source/f32` in the 50k cold samples, so it is an iteration candidate, not a default. |
| `index/turboquant` | Abandon current 111h high-recall candidate | Packets `023`, `024`, `027`, `028`, `029`, `040` | Centroid-relative TurboQuant was implemented and measured. It is the smaller compact quantized candidate in the default warm matrix, but it misses 0.97/0.99 at 50k/100k/1M and reaches only 0.9565 at 50k nprobe 200 in the cold diagnostic. Keep it out of the high-recall coarse-rerank product path unless a separate lower-recall compact product target is defined. |

## Requirement Audit

| Requirement | Evidence | Status |
| --- | --- | --- |
| Remove misleading query-time compact conversion from product-facing table placement. | `src/am/ec_ivf/options.rs` rejects compact `source`, reserves `table`, and keeps `source_diagnostic` explicit; packets `001`, `013`, `034`. | Satisfied. |
| Implement f16, RaBitQ4, RaBitQ8, and TurboQuant through one common architecture. | `src/am/ec_ivf/rerank.rs::RerankPayloadCodec`, `RerankEncoder`, `RerankScorer`; build/insert use `encode_with_centroid`; scan uses common source/index scoring surfaces; packets `002`, `022`, `023`, `032`, `035`, `037`. | Satisfied. |
| Use scorer-width packed index-side group/segment layout with payload-heavy continuation segments. | `0x2B` group header and `0x2C` payload segment codec in `page.rs`; build emits scorer-width groups; scan resolves by group header TID; packets `003`, `005`, `016`; reviewer feedback in `016/feedback/2026-06-20-01-reviewer.md`. | Satisfied. |
| Table-owned compact payloads implemented and measured, or explicitly replaced by evidence-backed storage design. | Packet `034` documents why `table` is reserved and why 111h uses `source/f32` plus `index/*` instead. | Satisfied by evidence-backed replacement. |
| PG18 correctness coverage proves payload consistency across build, insert, delete/vacuum, mixed old/new, update/snapshot visibility. | Packets `009`-`016` plus review feedback `016/feedback/2026-06-20-01-reviewer.md`; packet `030` audit; packet `031` update/snapshot fixture. Current test names include `test_ec_ivf_index_placement_insert_maintains_packed_group`, `mixed_fallback_chain`, `partial_final_group`, `vacuum_tombstones_packed_group_slot`, and `update_snapshot_payload`. | Satisfied for implemented `source` and `index` paths; `table` is rejected/reserved. |
| Durable layout changes bump format version and update fixtures/matrix. | Packed group layout advanced format version in packet `005`; residual/centroid and metadata-backed RaBitQ score/clip advanced through v8 in packets `022`, `023`, `037`. Current code has `EC_IVF_INDEX_FORMAT_VERSION = 8`, `fixtures/on-disk/ivf_metadata_v8.hex`, docs, size assertions, and upgrade matrix entries. | Satisfied. |
| Full matrix uses `ecaz bench suite`, with packet-local configs/manifests/raw results. | Post-v7 10k/50k/100k/1M packets `026`, `027`, `024`, `028`; cross-scale matched recall packet `029`; RaBitQ8 clip A/B packet `036`; 50k cold diagnostic packet `040`. | Satisfied with disclosed limitations: 1M uses a shared-table one-active-index-at-a-time surface; clip=4 has targeted 100k plus 50k evidence, not a full 10k/1M rerun. |
| Include recall, latency, storage, build, and read-amplification/stage metrics. | Suite result JSONL/logs in packets `024`, `026`, `027`, `028`, `036`, `040`; EXPLAIN/admin counter coverage audited in packet `030`. | Satisfied. |
| Final decision table covers f32 source, f16, RaBitQ4, RaBitQ8, and TurboQuant at matched recall. | Packet `029` warm matched-recall table; final decision table above incorporates packets `036` and `040` for the later RaBitQ8 clip/cold evidence. | Satisfied. |

## Evidence Map

- Packet `026`: post-v7 10k full format/width/nprobe suite, `completed=81 failed=0`.
- Packet `027`: post-v7 50k full format/width/nprobe suite, `completed=81 failed=0`.
- Packet `024`: post-v7 100k full format/width/nprobe suite with documented ENOSPC continuations.
- Packet `028`: post-v7 1M full format/width/nprobe suite on shared-table one-active-index surface.
- Packet `029`: cross-scale matched-recall decision table at targets 0.95/0.97/0.99.
- Packet `036`: RaBitQ8 score/clip A/B, proving clip=4 materially improves high-nprobe recall.
- Packet `037`: persists RaBitQ score/clip knobs into v8 metadata and closes the ALTER footgun.
- Packet `038`: closes copy/slab decision by implementation for f16/TurboQuant and benchmark-away rationale for RaBitQ4/8.
- Packet `040`: local 50k cold-start diagnostic over the final candidate set.

## Limits That Remain Outside 111h

- No remote-storage sweep was run. Packet `040` is a local OS page-cache eviction diagnostic using `posix_fadvise(DONTNEED)` and one latency sample per nprobe.
- A future table-owned compact payload feature requires a new PostgreSQL storage/MVCC design; 111h reserves `table` rather than implementing it.
- A future RaBitQ8 product iteration should start from clip=4 metadata-backed indexes and re-run a broader matrix only if product goals value index-only compact payloads despite source/f32 being smaller and faster in the current evidence.
- A future f16 iteration must solve the current storage footprint problem before promotion.

These limits do not block Task 111h closeout because the task's no-deferral requirement was to implement or evidence-reject every named format and placement for the current product decision. That decision is now explicit for every format.
