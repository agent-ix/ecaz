# Task 124 Phase 6 local cache eviction summary

- head SHA: `3be1ba32e94c2c33a4b222ee7a7271933b94e026`
- packet: `reviews/task-124/015-tq-phase6-local-cache-evict`
- host: local macOS PG18, socket `/Users/peter/.pgrx`, port `28818`
- cache eviction mode: `evicted_macos_f_nocache`
- purpose: preserve the relation-cache eviction evidence because the suite raw-step `--log-file` placeholders are zero-byte even though suite status marks both raw steps as succeeded.

## Commands

```text
target/release/ecaz --host /Users/peter/.pgrx --port 28818 dev evict-relation-cache --prefix task124_phase6_f32_100k
target/release/ecaz --host /Users/peter/.pgrx --port 28818 dev evict-relation-cache --prefix task124_phase6_tq_100k
```

## Key output

```text
cache_evict_start database=tqvector_bench dry_run=false data_directory=/Users/peter/.pgrx/data-18 relations=5 files=10 bytes=1690509312
cache_evict_file status=evicted_macos_f_nocache relation=task124_phase6_f32_100k_coarse_rerank_idx relkind=i bytes=23584768 path=/Users/peter/.pgrx/data-18/base/4906084/57392623
cache_evict_relation relation=task124_phase6_f32_100k_coarse_rerank_idx bytes=23584768
cache_evict_summary database=tqvector_bench dry_run=false relations=5 files=10 bytes=1690509312

cache_evict_start database=tqvector_bench dry_run=false data_directory=/Users/peter/.pgrx/data-18 relations=5 files=10 bytes=1772642304
cache_evict_file status=evicted_macos_f_nocache relation=task124_phase6_tq_100k_coarse_rerank_idx relkind=i bytes=105717760 path=/Users/peter/.pgrx/data-18/base/4906084/57693650
cache_evict_relation relation=task124_phase6_tq_100k_coarse_rerank_idx bytes=105717760
cache_evict_summary database=tqvector_bench dry_run=false relations=5 files=10 bytes=1772642304
```
