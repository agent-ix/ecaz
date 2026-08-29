# DistANN research pipeline candidates

> Compiled 29 Aug 2026 against a clone of agent-ix/ecaz @ main (ADR-085/086/087,
> DistANN task ledger 161-239, src/am/ec_distann/, reviews/ benchmark packets).
>
> Intake note (2026-08-29): candidates A–D below are filed as GitHub issues
> [#107](https://github.com/agent-ix/ecaz/issues/107),
> [#108](https://github.com/agent-ix/ecaz/issues/108),
> [#109](https://github.com/agent-ix/ecaz/issues/109),
> [#110](https://github.com/agent-ix/ecaz/issues/110) under research EPIC
> [#106](https://github.com/agent-ix/ecaz/issues/106) on
> [Project 19](https://github.com/orgs/agent-ix/projects/19). The SPIRE
> decision record is [#111](https://github.com/agent-ix/ecaz/issues/111);
> bibliography intake is [#112](https://github.com/agent-ix/ecaz/issues/112).

## What's missing from the DistANN bibliography

Read against the repo's actual ADRs, task ledger and open gates. Three papers
in your exact architecture family are cited nowhere in `spec/`, `plan/` or
`docs/` — and one of them sits precisely in the window ADR-085 D4 leaves open.

> **Calibration first**
>
> Your current transport share — roughly **4.1–5.0 ms of a 20–28 ms p50, so
> 15–25%** — is almost exactly where CoTra measures the empirical optimum for a
> distributed global graph (**22.5%**). Their reference points: a naive global
> RDMA index sits at 89.4% communication and is network-starved; independent
> scatter-gather sits below 1% and loses anyway on redundant compute. **Your
> remaining latency is not in transport**, which independently validates
> ADR-085 D4's deferral of BatANN and points at exactly where tasks 230–233 are
> already pointed.

## Three unreferenced papers, one of them load-bearing

I grepped `spec/`, `plan/` and `docs/` for the distributed-ANN literature.
DISTRIBUTEDANN appears in 18 files, SPANN in 34, BatANN in 8, SPFresh/LIRE in
3, Optimist in 2. These return zero:

| Paper | Why it matters here | Repo refs |
| --- | --- | --- |
| RED-ANNS (PVLDB 19(3) pp. 399–412) | **Same architecture family as DistANN** — one logical full graph across nodes, edges tagged local or remote. Four of its techniques are transport-agnostic. See below. | 0 |
| Gottesbüren et al. (PVLDB 18(6) 2025) | The definitive measurement of partitioning quality → locality. It is BatANN's partitioner. You shard by physical hash (task 179); this quantifies what that costs. | 0 |
| CoTra (arXiv 2507.06653) | Global proximity graph with *per-query* partition roles. Source of the comm/comp calibration above, and of a concrete data-vs-compute shipping threshold. | 0 |
| SPIRE (the paper) (arXiv 2512.17264) | Cited in your README for `ec_spire` — but worth re-reading now that you've walked away, because it's the strongest published form of the design you abandoned. Details below. | README only |

## RED-ANNS — four techniques that don't need RDMA

Fudan + Huawei, PVLDB 19(3). A single logical full graph distributed across
nodes; unlike BatANN the query does not migrate, the data is pulled to the
querying node. The RDMA is how they make remote reads cheap — but most of the
paper's leverage is in decisions made *before* a remote read is issued, which
is transport-independent.

### 1. Dependency-relaxed best-first search — and why it gets better at your latency

Best-first search is strictly sequential: iteration *i+1*'s frontier depends on
*i*'s distances, so every remote fetch stalls the CPU. Their Algorithm 1 splits
each expansion into local and remote neighbor batches, **issues async reads for
the remote batch and does not wait**, expands the local neighbors immediately,
and only polls for the remote results *n* iterations later. The cost is a
slightly longer search path; the gain is hidden latency.

Measured on MS-Turing 100M at recall@10 = 0.9: optimal at **n = 2**, where
**CPU wait time drops 90% and average query latency drops 37%**.

> **Why this is the find**
>
> The optimal relaxation depth scales with (remote latency ÷ per-iteration
> local work). RED-ANNS is hiding *microseconds* of RDMA at n=2. You are hiding
> **hundreds of microseconds to milliseconds** over the Postgres wire — which
> means a much larger n, more requests in flight, and more amortization. **The
> technique's value increases as transport latency increases.** And critically,
> it is not baton passing: it attacks the same hop-round stall that BatANN
> attacks, but it sits *below* ADR-085 D4's 50%-of-p50 trigger rather than
> above it. It is the thing to do while the trigger stays unmet.

Two cautions. The paper reports n=2 as optimal in ablation but says n=3
elsewhere without reconciling it. And the recall cost of large n is
characterised only qualitatively — you would need to measure the n-vs-recall
curve yourself at n ≫ 2, which is exactly the shape of a measurement packet.

### 2. Quantization-based pruning — 89% fewer remote fetches

Every node stores compressed codes for the *whole* dataset. Before fetching a
remote neighbor, compute an approximate distance locally and only fetch if it
is within ε of the current candidate pool's max. At ε = 1.0: **remote access
frequency drops 89%**; at ε = 1.2, 68%; below 1.0 recall degrades. Cost is
18.75% memory overhead at a 6.25% compression ratio.

Note the structural relationship to the paper you already work from:
DistributedANN embeds neighbor codes in the node entry, RED-ANNS replicates
the whole codebook everywhere. Same principle — **a local approximate-distance
oracle so the decision to fetch is made without a round trip** — different
placement. You have RaBitQ throughout; the question this raises is whether the
fetch decision itself currently consults it.

### 3. Similarity-weighted graph partitioning

METIS with edge weight `1 − (dist(u,v) − dist_min)/(dist_max − dist_min)`,
minimizing the weight of the cut. The insight is that **not all graph edges
are equally expensive to cut**: a Vamana graph deliberately contains
long-range shortcut edges from α-pruning, and those are traversed rarely,
while short edges are traversed constantly. Weighting the cut by similarity
makes the partitioner sacrifice exactly the edges you can afford to lose. An
unweighted min-cut can't tell the difference.

Effect: remote access ratio under **random placement is ~75%**; partitioning
plus duplication drives it to **10–16%**. That is a 5–7× reduction in remote
traffic from placement alone — the largest single lever in the paper.

### 4. Hot-vertex duplication at 4%

Offline profiling on a 1% query sample, purely frequency-ranked — not degree,
not centrality. Duplicating 4% of vertices absorbs **19–38% of remote
accesses** depending on dataset, cutting the remote ratio from 16% to 10% on
DEEP. Modest rather than transformative, and it is the empirical case for a
coordinator-side hot subgraph cache — which is what `crown_cache.rs` already
is, so this is a sizing and selection-policy reference rather than a new idea.

## Hash sharding versus graph partitioning

Task 179 puts you on physical hash shard generations — random placement,
following DistributedANN. That is a defensible choice and the two papers
genuinely disagree about it. But the disagreement is worth an explicit
decision record rather than an inherited default.

**The case for keeping hash.** DistributedANN shards randomly on purpose:
random placement immunizes against hot-shard skew, needs no placement
metadata, makes rebalancing trivial, and gives uniform load. At >1,000
machines they get p50 26 ms / p99 35 ms — a p99/p50 ratio of 1.35×, which is
remarkable for that fan-out and is largely *because* nothing is hot.

**The case against.** Gottesbüren et al. is the strongest counter-evidence,
and it contains one result that should change your priors:

> **The cheap-graph result**
>
> "Even low-quality graphs (k-NN graph recall ≈ 0.3) lead to high query
> recall: **more than 81% of top-10 neighbors are concentrated in one shard
> per query**" — at 40 shards, billion scale. You do not need an accurate k-NN
> graph to get excellent placement. A fast, sloppy one is enough. That removes
> the main practical objection to graph partitioning as placement metadata.

Their throughput over balanced k-means at 90% 10-recall: geometric mean 1.27×,
but **1.99× on Turing and 2.11× on SIFT1B**. At one shard probed, graph
partitioning gives +25% recall over balanced k-means on Turing. Build cost is
87–124 minutes at billion scale using KaMinPar with 5% imbalance tolerance.

For a hop-round architecture the relevant translation is not shard-probe
recall but **cross-node hop fraction**, and BatANN reports the number that
matters: with Gottesbüren partitioning on 10-server BIGANN-100M at 0.95
recall, **inter-partition hops are 24.3% of total hops**. Each avoided
cross-node hop is an avoided round trip. That is the direct lever on your
dominant cost term.

The honest synthesis: hash sharding buys tail predictability, graph
partitioning buys fewer round trips, and nobody has published the comparison
*for a hop-round global graph specifically*. Which is itself interesting —
see the last section.

## CoTra — per-query partition roles, and a threshold worth stealing

CoTra runs a global proximity graph but refuses to treat all partitions alike
for a given query. A replicated **navigation index** (1% sample, proximity
graph, on every machine) is searched first; its results classify each
partition as *primary* (many close accessed vectors — runs a full local
candidate queue, synchronized every 4 visited candidates) or *secondary*
(services computation requests on demand, no queue). It is the middle ground
between DistributedANN, where every node is a dumb scoring service, and
BatANN, where exactly one machine owns the query at a time.

The transplantable specific is their **data-versus-compute shipping rule**:
pull the vectors when a remote machine holds **≤ 2 neighbors** of the
currently visited node; push the task and return only distances otherwise.
The crossover is at two. That is a concrete heuristic for a decision your
expansion pushdown (task 205) already makes structurally.

Throughput at recall@10 ≥ 0.95 on 16 machines, SIFT100M: CoTra 116.6K QPS
against Shard 40.8K, Global 9.7K, Milvus 3.5K, single-machine 8-thread 30.3K.
Work overhead is ~20% more similarity computations than single-machine — the
price of relaxed parallel traversal, and the analogue of RED-ANNS's
lengthened search path.

**One caveat that matters for you:** CoTra is a throughput paper. It reports
no latency percentiles for itself at all — not mean, not p99. If latency is
your gate, it offers calibration and mechanisms but no target numbers.

## The SPIRE paper, now that you've left

Your README cites [arXiv 2512.17264](https://arxiv.org/html/2512.17264v1) for
`ec_spire`. Worth revisiting deliberately, because it is the strongest
published version of the architecture you abandoned, and it makes the
sharpest available argument *against* the global-graph thesis.

Their counter-claim: rather than reducing per-hop cost, **bound the hop count
structurally**. Recursive clustering until the top level fits one server's
memory gives O(levels) data-dependent network round trips — 3 levels at 1–2B,
4 at 8B. Each level is one round trip, fetching top-m partitions in parallel,
with storage nodes computing distances and returning top-N rather than raw
vectors. At 8B vectors on 46 production nodes they report under 20 ms
end-to-end at recall@5 = 0.9, and 9.64× throughput over DSPANN.

They also name your predecessor's failure mode precisely — *fidelity loss*,
vectors near partition boundaries being poorly represented by their centroid —
and characterize an optimal partition density around D ≈ 0.1 with a sharp
inflection. Task 139's finding that SPIRE scanned 93–95% of corpus
row-instances per query at `n128/b4`, with the nlists space only ever tested
at {32, 128, 1024}, reads very differently against that. It is possible SPIRE
died of a geometry that was never swept rather than of the architecture. That
does not argue for reviving it — DistANN's numbers are better and the program
has moved — but it is worth one paragraph in a decision record so the
abandonment rests on the right reason.

## Candidate work items

Framed against your open gates. Tasks 230–233 own storage layout, 209 owns
degraded completion, 228 owns the BatANN trigger — none of these overlap
those.

**Candidate A · highest leverage**

### Dependency-relaxed traversal (RED-ANNS §5.3)

Issue async remote expansions and consume them *n* hop-rounds later,
proceeding on local work meanwhile. Attacks the same sequential-stall cost as
BatANN without moving query state, so it is not blocked by ADR-085 D4 — and
unlike BatANN its payoff grows with transport latency rather than requiring
transport to already dominate.

- **Entry gate:** None new. Independent of 230–233; measurable on the current
  conforming owner path.
- **Measures:** The n-vs-recall-vs-latency curve at n ∈ {1,2,4,8,16}, at the
  shipped BW4/H100/L32 and at BW64/H8. The paper does not characterize recall
  cost at large n — that curve is the novel contribution.
- **Risk:** Longer search path costs recall; you are at 0.9280 on the shipped
  default, so headroom is thin. Pairs naturally with task 219's reopening
  trigger.

**Candidate B**

### Local approximate-distance gate on remote expansion

Before issuing a remote fetch, score the candidate against a locally-held
quantized code and skip it if it falls outside ε of the current pool maximum.
RED-ANNS measures 89% fewer remote accesses at ε = 1.0 for 18.75% memory. The
open question for you is placement: replicate a full codebook (RED-ANNS) or
embed neighbor codes in the node entry (DistributedANN).

- **Entry gate:** After 231/233 freeze the node block format — the
  neighbor-code variant is a format change.
- **Measures:** Remote fetch count and bytes per scan versus recall, across ε.
  Interacts with task 239's bounded read overfetch.
- **Note:** Check first whether the expansion path already consults RaBitQ
  before deciding to fetch. If it does, this is a tuning task, not a new
  mechanism.

**Candidate C · decision record**

### Placement policy: hash versus similarity-weighted partitioning

Task 179 inherited random hash sharding from DistributedANN's deliberate
choice. Gottesbüren's cheap-graph result (k-NN recall ≈ 0.3 suffices, 81%+
concentration at 40 shards) plus BatANN's 24.3% inter-partition hop fraction
make the alternative cheaper than it looks. Each avoided cross-node hop is an
avoided round trip.

- **Shape:** ADR, not a prototype. Record why hash was chosen, what graph
  partitioning would buy on hop fraction, and what it costs in rebalancing and
  tail predictability.
- **Measures:** If prototyped: cross-node hop fraction per query under hash
  versus weighted partitioning at fixed BW/H, plus per-node load skew. Nobody
  has published this comparison for a hop-round global graph.
- **Tension:** Hash buys the p99/p50 = 1.35× that DistributedANN reports.
  Locality may cost tail what it saves in mean — which is a real result
  either way.

**Candidate D · cheap**

### Comm/compute ratio as a standing suite metric

CoTra's three-point calibration (Global 89.4% starved, CoTra 22.5% optimal,
Shard <1% doing redundant work) turns a single ratio into a regime
diagnostic. You are computing the numerator already for task 194's transport
attribution; publishing it as a first-class metric would tell you at a glance
which failure mode any future configuration has fallen into.

- **Entry gate:** None. Rides task 237's EXPLAIN and suite-metric work.
- **Value:** Also the cleanest evidence for the 228 trigger — it is the same
  number ADR-085 D4 is thresholded on, expressed in a form with published
  comparison points.

## The field you're in is nearly empty

Across the entire VLDB 2026 research program the only vector-search sessions
are Hybrid Vector Search (filtered/predicate) and Hardware-Accelerated
Similarity Search (GPU/quantization). **There is not a single
distributed-graph-ANN paper in the VLDB 2026 main research track.** ICDE
2026's vector papers are all single-node — PRO-HNSW, BAMG, GLIDE, SINDI,
MINT. The global-graph work exists almost entirely as arXiv preprints
(DistributedANN, CoTra) plus one workshop poster (BatANN) and one PVLDB paper
you hadn't seen (RED-ANNS).

Combined with the finding from the earlier sweep — no PostgreSQL or pgvector
vector-search paper anywhere at VLDB 2026 — **"a single global graph sharded
across Postgres instances, with hop-round traversal inside an access method"**
is an unoccupied intersection of two unoccupied areas. Your task ledger
already contains more careful measurement of this architecture than most
published work: the BW/H regime sweep, the head scaling law, the wide-beam
A/B that overturned its own recommendation, the duplicate-ID audit that
qualified prior recall claims.

PVLDB Volume 20 takes submissions on the first of every month through March
2027, for VLDB 2027 in Athens. You are not late to this area.

## Method and caveats

Repo facts here come from a clone of `agent-ix/ecaz` at `main` — ADRs
085/086/087, the DistANN task ledger (161–239), `src/am/ec_distann/`, and
benchmark packets under `reviews/`. Bibliography gaps are a case-insensitive
grep of `spec/`, `plan/` and `docs/`; a paper discussed under a different
name than I searched for would show as a false gap.

RED-ANNS's n=2-versus-n=3 inconsistency is unresolved in the paper itself.
Its recall cost at large n is not characterized. CoTra publishes no latency
percentiles. HEXA (PVLDB 19(10) pp. 2922–2935) and Nova (PVLDB 19(12)
pp. 4453–4465) are confirmed to exist but their full text was not
retrievable — nothing about their contents is claimed here.
