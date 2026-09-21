# DISTRIBUTEDANN verbatim citations for the Task 205 review

Source: `DISTRIBUTEDANN: Efficient Scaling of a Single DISKANN Graph Across
Thousands of Computers`, arXiv:2509.06046, 8 pages. Local copy read at
DistributedANN (arXiv:2509.06046). Text extracted with `pypdf`;
line breaks and hyphenation are as extracted.

These are transcribed here so the review's citations are checkable from inside
the packet without re-extracting the PDF.

## §2.3 Algorithm 1 — Node Scoring Service

> **Input:** Node keys {k_i}, threshold score t, candidate limit l,
> full-dimension query q, SDC encoded query q_SDC
> **Static Data:** OPQ distance table
> **Output:** Sorted result IDs and distances R, sorted candidate IDs and
> distances C
>
> Initialize R←∅, C←∅
> Batch read the node entries n_i for all k_i
> **for all** n_i **do**
>   Compute d(q, v) for full-dimension vector v ∈ n_i and insert v into R
>   **for all** OPQ candidate p ∈ e_i **do**
>     **if** d_OPQ(q_SDC, p) < t **then**
>       Insert p into C
>     **end if**
>   **end for**
> **end for**
> Sort R and partial-sort C up to l
> Truncate C to l

Note the position of the final two lines: **outside** the `for all n_i` loop.
The sort and truncate happen once, over the candidate set merged across every
node in the batch.

## §2.3 bandwidth saving

> Additionally, since we only transmit scores over the network instead of full
> nodes, we achieve a bandwidth savings⁶ of
>
> ((1+R)(sizeof(id) + sizeof(score)) + d + d_OPQ) / ((1+R) sizeof(id) + d + R·d_OPQ)   (2)
>
> compared to a naive virtual disk approach.

Footnote 6:

> Using the same parameters as in Footnote 3, this is approximately a 6x saving.
> We increase the savings further by pruning any neighbors that are worse than
> the current worst member of the candidate heap before returning to the
> orchestration service.

The ~6× is the score-vs-node transmission saving. The threshold prune is
*additional* and unquantified.

## §2.4 Algorithm 2 — Orchestration Service

> **Input:** Full dimension query vector q. Beam width BW. Beam iterations
> (hops) H. Result count k. Head index result count k_head. **Candidate size
> L ≥ max(BW, k).**
> **Static Data:** OPQ distance table, OPQ codebooks
> **Output:** Sorted result IDs and distances R
>
> Initialize result heap H_R of size k, **candidate heap H_C of size L**.
> Encode OPQ query q_SDC using the codebooks.
> Search for k_head results in the head index, and insert into H_C
> **for** i = 1 **to** H **do**
>   **Let t = peek_worst(H_C)**
>   Take best BW candidates from H_C as keys K.
>   Let {R_i}, {C_i} = **NodeScoring(K, t, L, q, q_SDC)**.
>   Partially merge-sorted-lists of {R_i} up to k and {C_i} up to L, then insert
>   into respective heaps.
> **end for**
> Sort H_R into R

Two things this fixes about the ec_distann reading:

1. `H_C` is a heap **of size L**. `peek_worst` is the worst of the L retained
   candidates, not of an unbounded frontier.
2. The candidate limit passed to Algorithm 1 **is `L` itself** — the same
   constant every round. It is not a function of the round index or of remaining
   hops.

## §2.4 orchestration state

> Because this service has a small amount of persistent state, it can be hosted
> on many machines with low overhead, ensuring that the load is evenly
> distributed.

The bounded state is `H_R` (size k) and `H_C` (size L).

## §2.2 space amplification — accepted, not avoided

> Our first modification is based on the observation that, for a sufficiently
> large index, the array of compressed vectors will not be able to fit in a
> single machine. […] we instead decide to duplicate the compressed
> representation of each vector into all the graph nodes it is a neighbor of.
> This introduces a significant space amplification of
>
> ((1+R) sizeof(id) + d + R·d_OPQ) / (R sizeof(id) + d)   (1)

Footnote 3:

> For parameters of R=100, d=384, d_OPQ=64 and using 8-byte IDs instead of
> 4-byte IDs to allow an index of more than 4 billion vectors, this is
> approximately a 10x amplification.

## §2.2 head index

> We then build a conventional **sharded** in-memory ANN index over these
> vectors. We call this smaller index the head index.

## §4 production parameters and footprint

> The parameters for DISTRIBUTEDANN are H=5, BW=128, R=72, k=L=200, k_head=200,
> with a head index size of 2.5 billion vectors.

On a 50-billion-vector index, so the head is ~5% of N, not a constant.

Table 1 (DISTRIBUTEDANN vs Clustered Partitioning):

| Metric | DISTRIBUTEDANN | Clustered Partitioning |
|---|---:|---:|
| SSD Space (TiB) | 780 | 270 |
| Memory (TiB) | 42 | 18 |
| IO per query | 640 | 4800 |
| Network Bandwidth per query (MiB) | 1.4 | 0.3 |
| Throughput (QPS) | >100k | ~15k |

> The conventional index is bound by IO while DISTRIBUTEDANN is bound by SSD
> space and can continue scaling to over 100k QPS in the same footprint.

Title and §1: the graph is "spread across over a thousand machines."

## Derived quantities used in the review

Per-round candidate volume before truncation is `BW × R`:

- paper: 128 × 72 = 9,216, truncated to L=200 → **46× cut**
- ec_distann today: 4 × 32 = 128, against `l ≈ 350–396` → **no cut possible**
- ec_distann with L=200 (the paper's value): 128 candidates vs l=200 → still no
  cut. `L` must be chosen for the regime, not copied.
