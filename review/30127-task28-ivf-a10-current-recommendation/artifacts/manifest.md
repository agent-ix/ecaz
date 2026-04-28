# Artifacts Manifest

## Source packets

- head SHA: `61ecd74a`
- packet/topic: `30127-task28-ivf-a10-current-recommendation`
- lane: Task 28 IVF A10 quantizer recommendation synthesis
- measurement type: synthesis over packet-local artifacts; no new benchmark command
- cache state: source packets are warm local development unless noted otherwise

### 30084

- topic: `30084-task28-ivf-quantizer-headtohead-smoke`
- role: first TurboQuant / PQ-FastScan / RaBitQ smoke
- cited key lines:
  - TurboQuant 10k nprobe=48 recall@10 `1.0000`, p50 `82.6 ms`
  - PQ-FastScan initial shape 10k nprobe=48 recall@10 `0.3890`, p50 `40.2 ms`
  - RaBitQ 10k nprobe=32 recall@10 `0.9800`, narrowed p50 `1276.7 ms`

### 30091

- topic: `30091-task28-ivf-100k-pqfastscan-turboquant-comparison`
- role: same-fixture 100k PQ-FastScan g8 versus TurboQuant comparison
- cited key lines:
  - PQ-FastScan g8 nprobe=32 recall@10 `0.9930`, p50 `279.5 ms`, index size `18 MB`
  - TurboQuant nprobe=32 recall@10 `0.9930`, p50 `464.8 ms`, index size `87 MB`
  - PQ-FastScan g8 nprobe=48 recall@10 `1.0000`, p50 `407.6 ms`
  - TurboQuant nprobe=48 recall@10 `1.0000`, p50 `705.7 ms`

### 30097

- topic: `30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh`
- role: refreshed 10k/25k TurboQuant versus PQ-FastScan g8 comparison
- cited key lines:
  - 10k TurboQuant w750 recall@10/100 `1.0000/0.9966`, p50 `118.8 ms`, size `9416 kB`
  - 10k PQ-FastScan g8 w750 recall@10/100 `0.9910/0.9360`, p50 `85.4 ms`, size `2448 kB`
  - 25k TurboQuant w750 recall@10/100 `0.9990/0.9929`, p50 `231.5 ms`, size `22 MB`
  - 25k PQ-FastScan g8 w750 recall@10/100 `0.9940/0.9256`, p50 `145.7 ms`, size `5176 kB`

### 30126

- topic: `30126-task28-ivf-a9-100k-current-refresh`
- role: current-head 100k PQ-FastScan g8 selected point
- cited key lines:
  - build time `216788.531 ms`
  - index size `19,791,872` bytes
  - recall@10 `0.9920`, NDCG@10 `0.9997`
  - recall@100 `0.9552`, NDCG@100 `0.9983`
  - p50/p95/p99 `169.3/191.2/194.4 ms`
  - memory HWM `153816 kB`
