# Task 70 Packet 008 Artifact Manifest

- Code head measured: `f1665fe5760f8eec569bb2560841ed91f270ed09`
- Task bucket: `reviews/task-70/`
- Packet path: `reviews/task-70/008-frontier-subtiming-profile/`
- Timestamp: `2026-05-31T20:41:47Z`
- Lane: Task 70 Phase 2 profiling follow-up after packets 003, 004, and 005 feedback
- Fixture: real10K DBPedia staged corpus
- Storage format / rerank mode: `ec_diskann`, pq_fastscan, graph_degree=32, build_list_size=100, alpha=1.2, rerank_budget=64, top_k=10
- Isolation: one packet-local table/index prefix, `task70_008_diskann`; pgvectorscale compare rebuilds a separate packet-local compare table
- Runner: `ecaz bench suite`

## Suite Command

```sh
./target/debug/ecaz bench suite run --config reviews/task-70/008-frontier-subtiming-profile/artifacts/suite.json --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-70/008-frontier-subtiming-profile/artifacts/suite-manifest.json --results-output reviews/task-70/008-frontier-subtiming-profile/artifacts/results.jsonl --log-file reviews/task-70/008-frontier-subtiming-profile/artifacts/suite-run.log
```

Dry run was executed first with the same config and packet-local manifest/log outputs.

## Artifacts

| artifact | command / source | key result |
| --- | --- | --- |
| `suite.json` | Checked-in `SuiteConfig` for packet 008 | Defines load, recall, profiled latency, raw profile notices, EXPLAIN, and pgvectorscale compare steps. |
| `suite-dry-run.log`, `suite-dry-run-manifest.json` | `ecaz bench suite run --dry-run ...` | Dry run succeeded and kept expected outputs under this packet. |
| `install-ecaz-pg-test.log` | `./target/debug/ecaz dev install ecaz-pg-test --pg 18 --database tqvector_bench --log-file ...` | Installed the PG18 extension for code head `f1665fe57`. |
| `suite-run.log`, `suite-manifest.json`, `results.jsonl` | Suite command above | Suite succeeded; all selected steps exit code 0. |
| `load-diskann-real10k.log` | Suite load step | copy corpus 5.72s, encode corpus 2.35s, build index 7.27s, total 25.58s. |
| `recall-diskann-real10k-l64-l200.log` | Suite recall step | L64 recall 0.9965, mean q-time 0.68 ms; L200 recall 0.9975, mean q-time 0.96 ms. |
| `latency-diskann-real10k-l64-l200-profiled.log` | Suite latency step with `ec_diskann.scan_profile_notice=on` | L64 mean 0.83 ms, p95 1.13 ms, p99 1.60 ms; L200 mean 1.29 ms, p95 1.51 ms, p99 1.59 ms. |
| `profile-notices-diskann-real10k-l64.sql`, `profile-notices-diskann-real10k-l64.log` | Suite raw SQL profile step | 200 NOTICE rows; frontier mean 401.60 us; candidate-heap mean 4.13 us; visited-set mean 0.23 us; neighbor-iter mean 0.09 us. |
| `profile-notices-diskann-real10k-l200.sql`, `profile-notices-diskann-real10k-l200.log` | Suite raw SQL profile step | 200 NOTICE rows; frontier mean 920.04 us; candidate-heap mean 10.96 us; visited-set mean 1.06 us; neighbor-iter mean 0.79 us. |
| `explain-diskann-real10k-l64.sql`, `explain-diskann-real10k-l64.log` | Suite EXPLAIN step | Planner scan selection live; effective_list_size=64; execution time 0.962 ms. |
| `explain-diskann-real10k-l200.sql`, `explain-diskann-real10k-l200.log` | Suite EXPLAIN step | Planner scan selection live; effective_list_size=200; execution time 1.584 ms. |
| `compare-vectorscale-real10k-l64-l200.log` | Suite pgvectorscale compare step | L64 ec_diskann 0.69 ms mean / 0.9965 recall vs pgvectorscale 0.65 ms / 0.9960; L200 ec_diskann 0.97 ms / 0.9975 vs pgvectorscale 1.21 ms / 1.0000. |
| `truth-real10k-k10.json` | Recall truth cache | Packet-local ground truth cache for k=10. |
| `frontier-subtiming-summary.md` | Manual aggregation from `results.jsonl` and profile NOTICE logs | Summary tables and interpretation for review. |
| `cargo-test-diskann-scan.log` | `cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::` | 19 passed. |
| `cargo-check-pg18.log` | `cargo check --all-targets --no-default-features --features pg18` | Finished successfully. |

## Aggregation Command

The profile means and percentiles in `frontier-subtiming-summary.md` were computed from the two NOTICE logs with:

```sh
perl -we 'use strict; use warnings; my @keys=qw(setup_us entry_resolution_us graph_read_decode_us prefilter_score_us frontier_us frontier_candidate_heap_us frontier_visited_set_us frontier_neighbor_iter_us frontier_retained_insert_us heap_prefetch_us exact_rerank_us result_expand_us total_us graph_read_count prefilter_count frontier_candidate_heap_ops frontier_visited_set_ops frontier_neighbor_slots frontier_retained_inserts rerank_count result_count); for my $f (@ARGV){ open my $fh,"<",$f or die $!; my %v; my $n=0; while(my $line=<$fh>){ next unless $line =~ /ec_diskann_scan_profile/; $n++; while($line =~ /([a-zA-Z0-9_]+)=([0-9.]+)/g){ push @{$v{$1}}, $2+0; } } close $fh; print "FILE $f rows $n\n"; for my $k (@keys){ my @a=sort {$a<=>$b} @{$v{$k}||[]}; next unless @a; my $sum=0; $sum+=$_ for @a; my $mean=$sum/@a; my $p50=$a[int(0.50*($#a))]; my $p95=$a[int(0.95*($#a))]; my $p99=$a[int(0.99*($#a))]; printf "%s mean=%.2f p50=%.2f p95=%.2f p99=%.2f min=%.2f max=%.2f\n", $k,$mean,$p50,$p95,$p99,$a[0],$a[-1]; } print "\n"; }' reviews/task-70/008-frontier-subtiming-profile/artifacts/profile-notices-diskann-real10k-l64.log reviews/task-70/008-frontier-subtiming-profile/artifacts/profile-notices-diskann-real10k-l200.log
```
