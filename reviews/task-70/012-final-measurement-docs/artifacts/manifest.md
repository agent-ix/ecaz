# Task 70 Packet 012 Artifact Manifest

- Head SHA: `5aad1539eb285153a44d59147e7f12fdde737ebd`
- Code head measured: `1c0de8436e1a67421a7a00d94006123a06f2a302`
- Docs commit: `5f7f83b35b4b10755fd430e887013cb821e27fbb`
- Task bucket: `reviews/task-70/`
- Packet path: `reviews/task-70/012-final-measurement-docs/`
- Timestamp: `2026-05-31T21:12:15Z`
- Lane: Task 70 final closeout measurement and docs update
- Fixture: real10K DBPedia staged corpus
- Storage format / rerank mode: `ec_diskann`, `pq_fastscan`, `graph_degree=32`, `build_list_size=100`, `alpha=1.2`, `rerank_budget=64`, `top_k=10`
- Isolation: one packet-local table/index prefix, `task70_012_diskann`; pgvectorscale compare rebuilds a separate packet-local compare table
- Runner: `ecaz bench suite`

## Install Command

```sh
./target/debug/ecaz dev install ecaz-pg-test --pg 18 --database tqvector_bench --log-file reviews/task-70/012-final-measurement-docs/artifacts/install-ecaz-pg-test.log
```

Installed backend SHA256: `d1ee97a40d16f7a4925d27971c5c03ccfd4554be915b6b214d59c79958e1d979`.

## Suite Commands

Dry run:

```sh
./target/debug/ecaz bench suite run --config reviews/task-70/012-final-measurement-docs/artifacts/suite.json --dry-run --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-70/012-final-measurement-docs/artifacts/suite-dry-run-manifest.json --log-file reviews/task-70/012-final-measurement-docs/artifacts/suite-dry-run.log
```

Full run:

```sh
./target/debug/ecaz bench suite run --config reviews/task-70/012-final-measurement-docs/artifacts/suite.json --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-70/012-final-measurement-docs/artifacts/suite-manifest.json --results-output reviews/task-70/012-final-measurement-docs/artifacts/results.jsonl --log-file reviews/task-70/012-final-measurement-docs/artifacts/suite-run.log
```

## Artifacts

| artifact | command / source | key result |
| --- | --- | --- |
| `suite.json` | Checked-in `SuiteConfig` for packet 012 | Defines load, recall, profiled latency, raw profile notices, EXPLAIN, and pgvectorscale compare steps. |
| `suite-dry-run.log`, `suite-dry-run-manifest.json` | Dry-run command above | Dry run succeeded and kept expected outputs under this packet. |
| `install-ecaz-pg-test.log` | Install command above | Installed PG18 extension for the measured tree. |
| `suite-run.log`, `suite-manifest.json`, `results.jsonl` | Suite command above | Suite succeeded; all selected steps exit code 0. |
| `load-diskann-real10k.log` | Suite load step | copy corpus 5.76s, encode corpus 2.45s, build index 6.86s, total 25.36s. |
| `recall-diskann-real10k-l64-l200.log` | Suite recall step | L64 recall 0.9965, mean q-time 0.64 ms; L200 recall 0.9975, mean q-time 0.88 ms. |
| `latency-diskann-real10k-l64-l200-profiled.log` | Suite latency step with `ec_diskann.scan_profile_notice=on` | L64 mean 0.76 ms, p95 0.85 ms, p99 0.92 ms; L200 mean 1.24 ms, p95 1.43 ms, p99 1.51 ms. |
| `profile-notices-diskann-real10k-l64.sql`, `profile-notices-diskann-real10k-l64.log` | Suite raw SQL profile step | 200 NOTICE rows; frontier mean 366.23 us; total mean 475.19 us. |
| `profile-notices-diskann-real10k-l200.sql`, `profile-notices-diskann-real10k-l200.log` | Suite raw SQL profile step | 200 NOTICE rows; frontier mean 844.38 us; total mean 957.56 us. |
| `explain-diskann-real10k-l64.sql`, `explain-diskann-real10k-l64.log` | Suite EXPLAIN step | Planner scan selection live; effective_list_size=64; index_pages=603. |
| `explain-diskann-real10k-l200.sql`, `explain-diskann-real10k-l200.log` | Suite EXPLAIN step | Planner scan selection live; effective_list_size=200; index_pages=603. |
| `compare-vectorscale-real10k-l64-l200.log` | Suite pgvectorscale compare step | L64 ec_diskann 0.64 ms mean / 0.9965 recall vs pgvectorscale 0.60 ms / 0.9960; L200 ec_diskann 0.88 ms / 0.9975 vs pgvectorscale 1.14 ms / 1.0000. |
| `truth-real10k-k10.json` | Recall truth cache | Packet-local ground truth cache for k=10. |
| `final-summary.md` | Manual aggregation from `results.jsonl`, profile NOTICE logs, and docs diff | Summary tables and closeout interpretation for review. |

## Key Result Lines

- Recall floors: L64 `0.9965`; L200 `0.9975`.
- Clean compare: L64 `ec_diskann` mean `0.64 ms`, p99 `0.91 ms` vs pgvectorscale mean `0.60 ms`, p99 `0.89 ms`.
- Clean compare: L200 `ec_diskann` mean `0.88 ms`, p99 `1.13 ms` vs pgvectorscale mean `1.14 ms`, p99 `1.47 ms`.
- Phase split: L64 frontier mean `366.23 us`, exact rerank mean `87.07 us`, total mean `475.19 us`.
- Phase split: L200 frontier mean `844.38 us`, exact rerank mean `91.77 us`, total mean `957.56 us`.
- Docs: `docs/benchmarks.md` records the Task 70 cross-engine closeout row and residual gap/closure narrative.

## Aggregation Command

The profile means and percentiles in `final-summary.md` were computed from the two NOTICE logs with:

```sh
perl -we 'use strict; use warnings; my @keys=qw(setup_us entry_resolution_us graph_read_decode_us prefilter_score_us frontier_us frontier_candidate_heap_us frontier_visited_set_us frontier_neighbor_iter_us frontier_retained_insert_us heap_prefetch_us exact_rerank_us result_expand_us total_us graph_read_count prefilter_count frontier_candidate_heap_ops frontier_visited_set_ops frontier_neighbor_slots frontier_retained_inserts rerank_count result_count); for my $f (@ARGV){ open my $fh,"<",$f or die $!; my %v; my $n=0; while(my $line=<$fh>){ next unless $line =~ /ec_diskann_scan_profile/; $n++; while($line =~ /([a-zA-Z0-9_]+)=([0-9.]+)/g){ push @{$v{$1}}, $2+0; } } close $fh; print "FILE $f rows $n\n"; for my $k (@keys){ my @a=sort {$a<=>$b} @{$v{$k}||[]}; next unless @a; my $sum=0; $sum+=$_ for @a; my $mean=$sum/@a; my $p50=$a[int(0.50*($#a))]; my $p95=$a[int(0.95*($#a))]; my $p99=$a[int(0.99*($#a))]; printf "%s mean=%.2f p50=%.2f p95=%.2f p99=%.2f min=%.2f max=%.2f\n", $k,$mean,$p50,$p95,$p99,$a[0],$a[-1]; } print "\n"; }' reviews/task-70/012-final-measurement-docs/artifacts/profile-notices-diskann-real10k-l64.log reviews/task-70/012-final-measurement-docs/artifacts/profile-notices-diskann-real10k-l200.log
```

## Checksums

```text
21877f8751ed31ade9e86c3a5b69a058453e5f23  compare-vectorscale-real10k-l64-l200.log
817079f80bb96f50eb9f93b445e44d68759f6030  explain-diskann-real10k-l200.log
fb27b97549158674f17d2e29cc2f356f34337d38  explain-diskann-real10k-l200.sql
39084795a493464ef03a63cdf870e619f9a21569  explain-diskann-real10k-l64.log
bd43b8a3fd486059b328de2bfc09d17446bb664f  explain-diskann-real10k-l64.sql
9888c241b53f3f00bf5ecd53f33164d0f88cc21a  install-ecaz-pg-test.log
8dcfb0669549019d408500b8befd88f09d403221  latency-diskann-real10k-l64-l200-profiled.log
4b8b5014751da498474a8b0f57cc198ff1914fa8  load-diskann-real10k.log
1119d9c5645576af73e81f1ee462d5093d8f8cdb  profile-notices-diskann-real10k-l200.log
d03e146b2834aead3257cb70775dc7320339ef25  profile-notices-diskann-real10k-l200.sql
bd5ec62972416abae3993053ba9a9f537f99ef9c  profile-notices-diskann-real10k-l64.log
5028e433f2cdd04415334b5d12528022408c9c68  profile-notices-diskann-real10k-l64.sql
4abee9c54b8c43fff08a0550c460f9c16c822857  recall-diskann-real10k-l64-l200.log
00fb6c9bdc2dd27817108d19208921370bbe7c74  results.jsonl
c1daa88fb69dd9007279202722b7f6314c0caaf9  suite-dry-run-manifest.json
283ab3ba941a05eeb0016ed1c21914988f0deb42  suite-dry-run.log
51cc7a876988f64105a1bb7c5d097534a028c44a  suite-manifest.json
d30ee7a3ae1573396008f9b5a8abb28148b26b22  suite-run.log
d5559c4127b3593c435632030ea5894d447023a2  suite.json
841de9ed91811e825499494d6890cd3061c7c62b  truth-real10k-k10.json
```
