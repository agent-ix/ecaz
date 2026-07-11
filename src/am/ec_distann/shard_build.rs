//! FR-077 sharded closure-overlap build + stitch (Task 163 M1).
//!
//! Replaces the monolithic Vamana graph-construction core with:
//!   1. **Closure-overlap shard assignment** (`assign_shards`): spherical
//!      k-means over the corpus, then a distance-ratio closure band — a
//!      vector whose distance to a non-primary centroid is within
//!      `(1 + closure_epsilon)` of its best centroid distance is duplicated
//!      into that shard as well. The distance-ratio machinery ADR-085 cites
//!      lives on the unmerged `task-144-spire-closure-ratio-pruning` branch,
//!      so this is a fresh implementation against the already-plumbed
//!      `closure_epsilon` reloption (FR-077 "implement the ε band fresh").
//!   2. **Per-shard Vamana builds** (`build_shard_spool`): each shard is an
//!      independent seed-deterministic Vamana over the shared core
//!      (`build_vamana_graph_with_stats`), built in parallel; shard output
//!      adjacency is mapped back to the global vec-id space, sorted by global
//!      node id, and written to one sequential PostgreSQL-managed `BufFile` per
//!      shard.
//!   3. **Streaming stitch** (`stitch_shard_spool`): k-way merge lightweight
//!      shard cursors by global node (ADR-085 D8 — one node group + the prune
//!      working set held at a time), union the current group's neighbor lists,
//!      and `robust_prune` the union to at most `graph_degree` edges under the
//!      global distance. Cursor lookahead retains only the entry header; an
//!      adjacency payload is read only when its vec-id is the current group.
//!      A node that appears in exactly one shard passes through unchanged
//!      (stitch idempotence, FR-077-AC-2).
//!   4. **Reachability repair** (`repair_reachability`): a deterministic
//!      guard that guarantees FR-077-CON-3 (every node reachable from the
//!      entry medoid) by adding a single in-edge from the nearest reached
//!      node to each stranded node. At corpus scale this fires ~never; the
//!      repair count is reported in the stitch stats.
//!
//! The whole pipeline is deterministic under a fixed seed (FR-077): identical
//! corpus + seed + options yield an identical stitched graph — the invariant
//! the M2 single-vs-multinode result-identity test (FR-081-AC-1) depends on.
//!
//! The output is a `VamanaGraph` in the global node-id space, byte-identical
//! in shape to the monolithic build's output, so the record / directory /
//! head-sample staging in `ambuild.rs` is unchanged (FR-077 non-goal: no
//! head-index changes beyond consuming multi-shard entry samples).

use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};
use std::mem::size_of;
#[cfg(not(test))]
use std::ptr::NonNull;
use std::sync::mpsc::{sync_channel, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use pgrx::pg_sys;
#[cfg(test)]
use rayon::prelude::*;

use crate::am::common::training::train_spherical_kmeans;
use crate::am::ec_diskann::maybe_check_for_interrupts;
use crate::am::ec_diskann::source_inner_product_distance;
use crate::am::{
    approximate_medoid, bfs_reachable, build_vamana_graph_with_stats, robust_prune, Candidate,
    VamanaGraph,
};

/// Same medoid sample cap as the monolithic build (`ambuild.rs`).
const DISTANN_MEDOID_SAMPLE_CAP: usize = 1000;

/// k-means iterations for shard assignment. Shard boundaries only need to be
/// spatially coherent enough for closure overlap to stitch back to global
/// quality, so a modest iteration budget keeps build cost down; determinism
/// holds regardless of the count.
const DISTANN_SHARD_KMEANS_ITERATIONS: usize = 10;

/// Golden-ratio odd constant used to derive independent per-shard build seeds
/// from the base seed (keeps each shard's Vamana insertion order distinct yet
/// deterministic).
const DISTANN_SHARD_SEED_STRIDE: u64 = 0x9E37_79B9_7F4A_7C15;

/// Internal, temporary-only shard-spool entry header: global node id followed
/// by neighbor count, both little-endian u32. The payload is `count` little-
/// endian u32 neighbor ids. This is deliberately not a persisted index/wire
/// format; the `BufFile` is resource-owner scoped to the build.
const DISTANN_SHARD_SPILL_HEADER_BYTES: usize = 8;

/// `BufFileSeek`'s `whence` argument is the C `SEEK_SET` value. Keep the
/// constant local because PostgreSQL's generated bindings do not expose the C
/// stdio constants.
#[cfg(not(test))]
const DISTANN_BUFFILE_SEEK_SET: i32 = 0;

/// Keep the backend interrupt loop responsive while a pure-Rust shard build is
/// running on scoped workers. PostgreSQL APIs remain confined to the backend
/// thread; worker code never calls `CHECK_FOR_INTERRUPTS`.
const DISTANN_SHARD_COMPLETION_POLL: Duration = Duration::from_millis(50);

/// Conservative per-file allowance beyond PostgreSQL's `PGAlignedBlock`:
/// the private `BufFile` fields, its initial `File` slot, and palloc chunk
/// headers. PostgreSQL intentionally exposes `BufFile` as an opaque C type, so
/// Rust cannot use `sizeof(BufFile)`; 256 bytes exceeds the PG17/PG18 fixed
/// structures while keeping the manifest bound explicitly conservative.
const DISTANN_BUFFILE_FIXED_OVERHEAD_BYTES: usize = 256;

/// Build-time diagnostics surfaced to the epoch/packet manifest (FR-077-AC-3,
/// ADR-085 D8). All counts are over the global node space.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct ShardBuildStats {
    /// Number of build shards actually used.
    pub(super) shard_count: usize,
    /// Sum of shard memberships divided by node count (1.0 = no overlap).
    pub(super) duplication_factor: f64,
    /// Largest single shard membership (load-balance sanity).
    pub(super) max_shard_size: usize,
    /// Total unioned candidate edges considered across the stitch, before
    /// `robust_prune`.
    pub(super) stitch_edges_before_prune: u64,
    /// Total edges emitted after `robust_prune`.
    pub(super) stitch_edges_after_prune: u64,
    /// Largest de-duplicated single-node neighbor union.
    pub(super) stitch_peak_union_len: usize,
    /// Total encoded bytes written to the PostgreSQL-managed shard spool.
    pub(super) shard_output_spill_bytes: u64,
    /// Peak retained bytes for every shard BufFile block buffer plus cursor and
    /// merge-heap metadata. Cursor lookahead contains no adjacency payload.
    pub(super) stitch_peak_cursor_bytes: usize,
    /// Peak retained bytes for the current vec-id's passthrough/union and the
    /// candidate/result capacity used by `robust_prune`.
    pub(super) stitch_peak_group_bytes: usize,
    /// Peak cursor + current-group/prune bytes. This is the D8/CON-4 manifest
    /// row; it excludes the required output graph and the source vectors that
    /// pre-exist the stitch, and includes every `BufFile` block buffer.
    pub(super) stitch_peak_retained_bytes: usize,
    /// Reachability repairs applied (FR-077-CON-3 guard); expected 0 at scale.
    pub(super) reachability_repairs: usize,
}

/// A single shard's Vamana output, projected into the global node-id space.
/// `nodes` is sorted ascending so the stitch can k-way-merge by node id and
/// hold only one node group at a time (ADR-085 D8).
struct ShardGraph {
    /// Global node ids that belong to this shard, ascending.
    nodes: Vec<u32>,
    /// Parallel to `nodes`: each entry is the shard-local Vamana adjacency of
    /// that node, remapped to global node ids.
    neighbors: Vec<Vec<u32>>,
}

#[cfg(not(test))]
struct ShardSpoolIo {
    file: NonNull<pg_sys::BufFile>,
}

#[cfg(not(test))]
impl ShardSpoolIo {
    fn create() -> Result<Self, String> {
        // SAFETY: called on the PostgreSQL backend's main thread while an
        // ambuild resource owner is active. `false` scopes the file to the
        // current transaction; Drop closes it earlier on the normal path.
        let file = unsafe { pg_sys::BufFileCreateTemp(false) };
        let file = NonNull::new(file)
            .ok_or_else(|| "ec_distann could not create shard BufFile".to_owned())?;
        Ok(Self { file })
    }

    fn rewind(&mut self) -> Result<(), String> {
        // SAFETY: `file` is live; every shard owns one BufFile and starts at
        // logical file 0, offset 0.
        let status =
            unsafe { pg_sys::BufFileSeek(self.file.as_ptr(), 0, 0, DISTANN_BUFFILE_SEEK_SET) };
        if status == 0 {
            Ok(())
        } else {
            Err("ec_distann could not rewind shard BufFile".to_owned())
        }
    }

    fn write_all(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        // SAFETY: BufFileWrite either consumes the full slice or raises a
        // PostgreSQL error; the slice remains live for the call.
        unsafe {
            pg_sys::BufFileWrite(
                self.file.as_ptr(),
                bytes.as_ptr().cast::<std::ffi::c_void>(),
                bytes.len(),
            );
        }
    }

    fn read_exact(&mut self, bytes: &mut [u8]) -> Result<(), String> {
        let mut read = 0_usize;
        while read < bytes.len() {
            // SAFETY: the remaining part of `bytes` is writable and the
            // BufFile pointer remains live.
            let count = unsafe {
                pg_sys::BufFileRead(
                    self.file.as_ptr(),
                    bytes[read..].as_mut_ptr().cast::<std::ffi::c_void>(),
                    bytes.len() - read,
                )
            };
            if count == 0 {
                return Err("ec_distann truncated shard BufFile entry".to_owned());
            }
            read += count;
        }
        Ok(())
    }

    fn ensure_eof(&mut self) -> Result<(), String> {
        let mut byte = 0_u8;
        // SAFETY: `byte` is a live one-byte output buffer and `file` remains
        // owned by this cursor. A zero-length read result is the BufFile EOF
        // contract; any byte proves the declared entry stream left a tail.
        let count = unsafe {
            pg_sys::BufFileRead(
                self.file.as_ptr(),
                (&mut byte as *mut u8).cast::<std::ffi::c_void>(),
                1,
            )
        };
        if count == 0 {
            Ok(())
        } else {
            Err("ec_distann shard spool has unconsumed trailing bytes".to_owned())
        }
    }
}

#[cfg(not(test))]
impl Drop for ShardSpoolIo {
    fn drop(&mut self) {
        // A PostgreSQL ereport can unwind through Rust. Closing a BufFile may
        // itself ereport (for example after temp-file failure), which would
        // become a panic-in-Drop backend abort. During unwind the current
        // resource owner will release the temporary file, so deliberately
        // leave it to PostgreSQL instead of risking a nested error.
        if std::thread::panicking() {
            return;
        }
        // SAFETY: this object uniquely owns the BufFile and closes it once.
        unsafe { pg_sys::BufFileClose(self.file.as_ptr()) };
    }
}

/// Unit tests exercise the same encoder and cursor merge without requiring a
/// live PostgreSQL backend. Runtime and pg_test builds use the BufFile version
/// above; this in-memory implementation is never part of extension code.
#[cfg(test)]
struct ShardSpoolIo {
    bytes: Vec<u8>,
    position: usize,
}

#[cfg(test)]
impl ShardSpoolIo {
    fn create() -> Result<Self, String> {
        Ok(Self {
            bytes: Vec::new(),
            position: 0,
        })
    }

    fn rewind(&mut self) -> Result<(), String> {
        self.position = 0;
        Ok(())
    }

    fn write_all(&mut self, bytes: &[u8]) {
        let end = self.position + bytes.len();
        if end > self.bytes.len() {
            self.bytes.resize(end, 0);
        }
        self.bytes[self.position..end].copy_from_slice(bytes);
        self.position = end;
    }

    fn read_exact(&mut self, bytes: &mut [u8]) -> Result<(), String> {
        let end = self.position + bytes.len();
        if end > self.bytes.len() {
            return Err("ec_distann truncated test shard spool entry".to_owned());
        }
        bytes.copy_from_slice(&self.bytes[self.position..end]);
        self.position = end;
        Ok(())
    }

    fn ensure_eof(&mut self) -> Result<(), String> {
        if self.position == self.bytes.len() {
            Ok(())
        } else {
            Err("ec_distann test shard spool has unconsumed trailing bytes".to_owned())
        }
    }
}

struct EncodedShard {
    bytes: Vec<u8>,
    entry_count: usize,
}

struct ShardSpill {
    io: ShardSpoolIo,
    entry_count: usize,
}

struct ShardSpool {
    spills: Vec<Option<ShardSpill>>,
    bytes_written: u64,
}

impl ShardSpool {
    fn create(shard_count: usize) -> Result<Self, String> {
        Ok(Self {
            spills: (0..shard_count).map(|_| None).collect(),
            bytes_written: 0,
        })
    }

    fn append_shard(
        &mut self,
        shard_index: usize,
        graph: ShardGraph,
        graph_degree: usize,
    ) -> Result<(), String> {
        let encoded = encode_shard(shard_index, graph, graph_degree)?;
        self.append_encoded_shard(shard_index, encoded)
    }

    /// Write a worker-produced shard encoding on the PostgreSQL backend
    /// thread. `EncodedShard` contains no nested adjacency allocations, so a
    /// completed `ShardGraph` has already been consumed and released before
    /// this method is reached.
    fn append_encoded_shard(
        &mut self,
        shard_index: usize,
        encoded: EncodedShard,
    ) -> Result<(), String> {
        if shard_index >= self.spills.len() || self.spills[shard_index].is_some() {
            return Err(format!(
                "ec_distann duplicate or invalid shard spool index {shard_index}"
            ));
        }
        let mut io = ShardSpoolIo::create()?;
        io.write_all(&encoded.bytes);
        let shard_bytes = u64::try_from(encoded.bytes.len())
            .map_err(|_| "ec_distann shard spill byte count exceeds u64".to_owned())?;
        self.bytes_written = self
            .bytes_written
            .checked_add(shard_bytes)
            .ok_or_else(|| "ec_distann total shard spill byte count overflow".to_owned())?;
        self.spills[shard_index] = Some(ShardSpill {
            io,
            entry_count: encoded.entry_count,
        });
        Ok(())
    }
}

/// Validate and consume a completed graph into one flat encoding while still
/// on its scoped worker. Returning a flat buffer rather than `ShardGraph`
/// prevents the backend completion queue from ever retaining a batch of
/// nested, complete adjacency graphs.
fn encode_shard(
    shard_index: usize,
    graph: ShardGraph,
    graph_degree: usize,
) -> Result<EncodedShard, String> {
    if graph.nodes.len() != graph.neighbors.len() {
        return Err(format!(
            "ec_distann shard {shard_index} node/adjacency length mismatch: {} vs {}",
            graph.nodes.len(),
            graph.neighbors.len()
        ));
    }
    if graph.nodes.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(format!(
            "ec_distann shard {shard_index} output is not strictly node-sorted"
        ));
    }

    let entry_count = graph.nodes.len();
    let payload_words = graph.neighbors.iter().try_fold(0_usize, |total, neighbors| {
        if neighbors.len() > graph_degree {
            return Err(format!(
                "ec_distann shard {shard_index} has an adjacency with {} neighbors, exceeding R={graph_degree}",
                neighbors.len()
            ));
        }
        total
            .checked_add(neighbors.len())
            .ok_or_else(|| "ec_distann shard neighbor count overflow".to_owned())
    })?;
    let header_bytes = entry_count
        .checked_mul(DISTANN_SHARD_SPILL_HEADER_BYTES)
        .ok_or_else(|| "ec_distann shard header byte count overflow".to_owned())?;
    let payload_bytes = payload_words
        .checked_mul(size_of::<u32>())
        .ok_or_else(|| "ec_distann shard payload byte count overflow".to_owned())?;
    let encoded_capacity = header_bytes
        .checked_add(payload_bytes)
        .ok_or_else(|| "ec_distann shard encoded byte count overflow".to_owned())?;
    let mut bytes = Vec::with_capacity(encoded_capacity);

    for (node, neighbors) in graph.nodes.into_iter().zip(graph.neighbors) {
        let neighbor_count = u32::try_from(neighbors.len())
            .map_err(|_| "ec_distann shard neighbor count exceeds u32".to_owned())?;
        bytes.extend_from_slice(&node.to_le_bytes());
        bytes.extend_from_slice(&neighbor_count.to_le_bytes());
        for neighbor in neighbors {
            bytes.extend_from_slice(&neighbor.to_le_bytes());
        }
    }
    debug_assert_eq!(bytes.len(), encoded_capacity);
    Ok(EncodedShard { bytes, entry_count })
}

#[derive(Debug, Clone, Copy)]
struct ShardEntryHeader {
    node: u32,
    neighbor_count: usize,
}

struct ShardCursor {
    io: ShardSpoolIo,
    remaining_entries: usize,
    current: Option<ShardEntryHeader>,
}

impl ShardCursor {
    fn open(mut spill: ShardSpill, node_count: usize, graph_degree: usize) -> Result<Self, String> {
        spill.io.rewind()?;
        let current = if spill.entry_count == 0 {
            None
        } else {
            Some(read_shard_entry_header(
                &mut spill.io,
                node_count,
                graph_degree,
            )?)
        };
        Ok(Self {
            io: spill.io,
            remaining_entries: spill.entry_count,
            current,
        })
    }

    fn consume_into(
        &mut self,
        node_count: usize,
        graph_degree: usize,
        neighbors: &mut Vec<u32>,
    ) -> Result<u32, String> {
        let header = self
            .current
            .take()
            .ok_or_else(|| "ec_distann consumed an exhausted shard cursor".to_owned())?;

        let payload_bytes = header
            .neighbor_count
            .checked_mul(size_of::<u32>())
            .ok_or_else(|| "ec_distann shard payload length overflow".to_owned())?;
        neighbors.clear();
        neighbors.resize(header.neighbor_count, 0);
        if payload_bytes > 0 {
            // SAFETY: `neighbors` is initialized and owns at least
            // `payload_bytes` writable bytes. Reading into its byte view and
            // converting every word from little endian preserves validity.
            let bytes = unsafe {
                std::slice::from_raw_parts_mut(neighbors.as_mut_ptr().cast::<u8>(), payload_bytes)
            };
            self.io.read_exact(bytes)?;
            for neighbor in neighbors.iter_mut() {
                *neighbor = u32::from_le(*neighbor);
                if *neighbor as usize >= node_count {
                    return Err(format!(
                        "ec_distann shard spool neighbor {} outside node count {node_count}",
                        *neighbor
                    ));
                }
            }
        }

        self.remaining_entries = self
            .remaining_entries
            .checked_sub(1)
            .ok_or_else(|| "ec_distann shard cursor entry underflow".to_owned())?;
        if self.remaining_entries > 0 {
            let next = read_shard_entry_header(&mut self.io, node_count, graph_degree)?;
            if next.node <= header.node {
                return Err(format!(
                    "ec_distann shard spool is not strictly sorted: {} after {}",
                    next.node, header.node
                ));
            }
            self.current = Some(next);
        }
        Ok(header.node)
    }
}

fn read_shard_entry_header(
    io: &mut ShardSpoolIo,
    node_count: usize,
    graph_degree: usize,
) -> Result<ShardEntryHeader, String> {
    let mut header = [0_u8; DISTANN_SHARD_SPILL_HEADER_BYTES];
    io.read_exact(&mut header)?;
    let node = u32::from_le_bytes(header[0..4].try_into().expect("four-byte node"));
    let neighbor_count =
        u32::from_le_bytes(header[4..8].try_into().expect("four-byte neighbor count")) as usize;
    if node as usize >= node_count {
        return Err(format!(
            "ec_distann shard spool node {node} outside node count {node_count}"
        ));
    }
    if neighbor_count > graph_degree {
        return Err(format!(
            "ec_distann shard spool node {node} has {neighbor_count} neighbors, exceeding R={graph_degree}"
        ));
    }
    Ok(ShardEntryHeader {
        node,
        neighbor_count,
    })
}

fn retained_cursor_bytes(
    shard_count: usize,
    cursor_capacity: usize,
    heap_capacity: usize,
) -> Result<usize, String> {
    let bytes_per_buffile = (pg_sys::BLCKSZ as usize)
        .checked_add(DISTANN_BUFFILE_FIXED_OVERHEAD_BYTES)
        .ok_or_else(|| "ec_distann per-BufFile accounting overflow".to_owned())?;
    let block_buffers = shard_count
        .checked_mul(bytes_per_buffile)
        .ok_or_else(|| "ec_distann shard BufFile accounting overflow".to_owned())?;
    let cursor_bytes = cursor_capacity
        .checked_mul(size_of::<ShardCursor>())
        .ok_or_else(|| "ec_distann shard cursor accounting overflow".to_owned())?;
    let heap_bytes = heap_capacity
        .checked_mul(size_of::<Reverse<(u32, usize)>>())
        .ok_or_else(|| "ec_distann shard heap accounting overflow".to_owned())?;
    block_buffers
        .checked_add(cursor_bytes)
        .and_then(|total| total.checked_add(heap_bytes))
        .ok_or_else(|| "ec_distann retained cursor accounting overflow".to_owned())
}

/// Result of closure-overlap shard assignment.
struct ShardAssignment {
    /// Per shard: the global node ids assigned to it (primary + closure), each
    /// list ascending.
    members: Vec<Vec<u32>>,
    duplication_factor: f64,
    max_shard_size: usize,
}

/// Pick the shard count for `node_count` nodes given the requested reloption
/// (`0` = auto). Auto scales gently with corpus size so shards stay large
/// enough for a coherent per-shard Vamana while still parallelizing big
/// builds; `1` (or any explicit request) is honored verbatim, which keeps the
/// monolithic fallback selectable (FR-077 / ADR-085 Consequences).
pub(super) fn resolve_shard_count(node_count: usize, requested: usize) -> usize {
    if requested >= 1 {
        return requested.min(node_count.max(1));
    }
    if node_count <= 20_000 {
        1
    } else {
        // ~one shard per 25k rows, capped so per-shard builds stay meaty.
        (node_count / 25_000).clamp(2, 16)
    }
}

/// Full sharded build. Returns a global-space `VamanaGraph`, the global entry
/// medoid, and manifest stats. Callers guarantee `shard_count >= 2`; the
/// single-shard case stays on the monolithic path in `ambuild.rs`.
pub(super) fn build_sharded_graph(
    source_refs: &[&[f32]],
    dimensions: usize,
    graph_degree: usize,
    build_list_size: usize,
    alpha: f32,
    shard_count: usize,
    closure_epsilon: f32,
    seed: u64,
) -> Result<(VamanaGraph, u32, ShardBuildStats), String> {
    let node_count = source_refs.len();
    debug_assert!(shard_count >= 2, "monolithic build handled by caller");

    let dist = |left: u32, right: u32| -> f32 {
        source_inner_product_distance(source_refs[left as usize], source_refs[right as usize])
    };

    // Global entry medoid — identical rule to the monolithic build so the
    // stitched graph shares an entry point with the fallback path.
    let medoid = approximate_medoid(node_count, DISTANN_MEDOID_SAMPLE_CAP, seed, dist);

    let ShardAssignment {
        members,
        duplication_factor,
        max_shard_size,
    } = assign_shards(source_refs, dimensions, shard_count, closure_epsilon, seed)?;
    let mut shard_spool =
        build_shard_spool(members, graph_degree, build_list_size, alpha, seed, &dist)?;

    let (graph, mut stats) =
        stitch_shard_spool(node_count, &mut shard_spool, graph_degree, alpha, &dist)?;
    stats.shard_count = shard_count;
    stats.duplication_factor = duplication_factor;
    stats.max_shard_size = max_shard_size;

    let mut graph = graph;
    stats.reachability_repairs = repair_reachability(&mut graph, medoid, graph_degree, &dist)?;

    Ok((graph, medoid, stats))
}

/// Spherical k-means + closure-overlap band. A node is assigned to its nearest
/// centroid (primary) and to every centroid whose `1 - ip` distance is within
/// `(1 + closure_epsilon)` of the primary distance (FR-077 closure overlap).
fn assign_shards(
    source_refs: &[&[f32]],
    dimensions: usize,
    shard_count: usize,
    closure_epsilon: f32,
    seed: u64,
) -> Result<ShardAssignment, String> {
    let node_count = source_refs.len();
    let model = train_spherical_kmeans(
        "ec_distann sharded build",
        source_refs,
        dimensions,
        shard_count,
        seed,
        DISTANN_SHARD_KMEANS_ITERATIONS,
    )?;
    let centroids = &model.centroids;
    let ratio = 1.0_f32 + closure_epsilon.max(0.0);

    let mut members: Vec<Vec<u32>> = vec![Vec::new(); shard_count];
    let mut centroid_distances = Vec::with_capacity(shard_count);
    let mut total_memberships: u64 = 0;
    for (node, source) in source_refs.iter().enumerate() {
        maybe_check_for_interrupts();
        // Distance to every centroid; primary = argmin.
        centroid_distances.clear();
        let mut best = f32::INFINITY;
        for centroid in centroids.iter() {
            let d = source_inner_product_distance(source, centroid);
            centroid_distances.push(d);
            if d < best {
                best = d;
            }
        }
        let band = best * ratio;
        let mut assigned_any = false;
        for (shard, &d) in centroid_distances.iter().enumerate() {
            // `<= band` includes the primary (d == best) and every centroid in
            // the closure band. `band` is finite and >= best, so at least the
            // primary always qualifies.
            if d <= band {
                members[shard].push(node as u32);
                total_memberships += 1;
                assigned_any = true;
            }
        }
        // Defensive: floating-point could in principle exclude all shards if
        // `best` were non-finite; keep every node in at least its primary.
        if !assigned_any {
            let primary = nearest_centroid_index(source, centroids);
            members[primary].push(node as u32);
            total_memberships += 1;
        }
    }

    // `members` are already ascending (nodes pushed in increasing order).
    let max_shard_size = members.iter().map(Vec::len).max().unwrap_or(0);
    let duplication_factor = if node_count == 0 {
        1.0
    } else {
        total_memberships as f64 / node_count as f64
    };

    Ok(ShardAssignment {
        members,
        duplication_factor,
        max_shard_size,
    })
}

fn nearest_centroid_index(source: &[f32], centroids: &[Vec<f32>]) -> usize {
    let mut best = f32::INFINITY;
    let mut best_index = 0;
    for (index, centroid) in centroids.iter().enumerate() {
        let d = source_inner_product_distance(source, centroid);
        if d < best {
            best = d;
            best_index = index;
        }
    }
    best_index
}

/// Build independent Vamana shards in parallel and immediately spill each
/// completion to a sequential PostgreSQL-managed BufFile. Workers consume
/// their completed `ShardGraph` into one flat encoding before sending it to a
/// bounded completion queue; the backend drains and writes completions as they
/// arrive. The queue therefore never retains the old batch-wide
/// `Vec<ShardGraph>`, and PostgreSQL I/O remains confined to the backend
/// thread. Parallel and sequential builds are identical because each shard is
/// pure and seeded. Membership lists are consumed by the workers and no shard
/// graph or assignment remains resident when the stitch starts.
fn build_shard_spool<D>(
    members: Vec<Vec<u32>>,
    graph_degree: usize,
    build_list_size: usize,
    alpha: f32,
    seed: u64,
    dist: &D,
) -> Result<ShardSpool, String>
where
    D: Fn(u32, u32) -> f32 + Sync,
{
    let shard_count = members.len();
    let mut spool = ShardSpool::create(shard_count)?;
    if shard_count == 0 {
        return Ok(spool);
    }

    let worker_count = rayon::current_num_threads().min(shard_count).max(1);
    if worker_count == 1 {
        for (shard_index, global_ids) in members.into_iter().enumerate() {
            maybe_check_for_interrupts();
            let graph = build_one_shard(
                shard_index,
                &global_ids,
                graph_degree,
                build_list_size,
                alpha,
                seed,
                dist,
            );
            let encoded = encode_shard(shard_index, graph, graph_degree)?;
            spool.append_encoded_shard(shard_index, encoded)?;
        }
        return Ok(spool);
    }

    let jobs = Arc::new(Mutex::new(
        members.into_iter().enumerate().collect::<VecDeque<_>>(),
    ));
    let (sender, receiver) = sync_channel::<(usize, Result<EncodedShard, String>)>(worker_count);
    std::thread::scope(|scope| -> Result<(), String> {
        for _ in 0..worker_count {
            let sender = sender.clone();
            let jobs = Arc::clone(&jobs);
            scope.spawn(move || loop {
                let job = {
                    let mut jobs = match jobs.lock() {
                        Ok(jobs) => jobs,
                        Err(_) => return,
                    };
                    jobs.pop_front()
                };
                let Some((shard_index, global_ids)) = job else {
                    return;
                };
                let graph = build_one_shard(
                    shard_index,
                    &global_ids,
                    graph_degree,
                    build_list_size,
                    alpha,
                    seed,
                    dist,
                );
                let result = encode_shard(shard_index, graph, graph_degree);
                // A send failure means the backend is already unwinding; no
                // PostgreSQL API or panic belongs on this worker thread.
                if sender.send((shard_index, result)).is_err() {
                    return;
                }
            });
        }
        drop(sender);

        let mut received = 0_usize;
        let mut first_error: Option<String> = None;
        while received < shard_count {
            match receiver.recv_timeout(DISTANN_SHARD_COMPLETION_POLL) {
                Ok((shard_index, result)) => {
                    received += 1;
                    maybe_check_for_interrupts();
                    match result {
                        Ok(encoded) if first_error.is_none() => {
                            if let Err(error) = spool.append_encoded_shard(shard_index, encoded) {
                                first_error = Some(error);
                            }
                        }
                        Ok(_encoded) => {
                            // Keep draining after the first error so bounded
                            // worker sends cannot deadlock the scoped workers.
                        }
                        Err(error) if first_error.is_none() => first_error = Some(error),
                        Err(_) => {}
                    }
                }
                Err(RecvTimeoutError::Timeout) => maybe_check_for_interrupts(),
                Err(RecvTimeoutError::Disconnected) => {
                    if first_error.is_none() {
                        first_error = Some(format!(
                            "ec_distann shard completion channel closed after {received} of {shard_count} shards"
                        ));
                    }
                    break;
                }
            }
        }
        first_error.map_or(Ok(()), Err)
    })?;
    Ok(spool)
}

#[cfg(test)]
fn build_shard_graphs<D>(
    members: &[Vec<u32>],
    graph_degree: usize,
    build_list_size: usize,
    alpha: f32,
    seed: u64,
    dist: &D,
) -> Vec<ShardGraph>
where
    D: Fn(u32, u32) -> f32 + Sync,
{
    members
        .par_iter()
        .enumerate()
        .map(|(shard_index, global_ids)| {
            build_one_shard(
                shard_index,
                global_ids,
                graph_degree,
                build_list_size,
                alpha,
                seed,
                dist,
            )
        })
        .collect()
}

fn build_one_shard<D>(
    shard_index: usize,
    global_ids: &[u32],
    graph_degree: usize,
    build_list_size: usize,
    alpha: f32,
    seed: u64,
    dist: &D,
) -> ShardGraph
where
    D: Fn(u32, u32) -> f32,
{
    let shard_len = global_ids.len();
    if shard_len == 0 {
        return ShardGraph {
            nodes: Vec::new(),
            neighbors: Vec::new(),
        };
    }
    // Shard-local distance maps local index -> global id -> global distance.
    let local_dist = |left: u32, right: u32| -> f32 {
        dist(global_ids[left as usize], global_ids[right as usize])
    };
    // Independent per-shard seed keeps each shard's insertion order distinct
    // yet deterministic.
    let shard_seed =
        seed.wrapping_add((shard_index as u64).wrapping_mul(DISTANN_SHARD_SEED_STRIDE));
    let medoid = approximate_medoid(shard_len, DISTANN_MEDOID_SAMPLE_CAP, shard_seed, local_dist);
    let (graph, _stats) = build_vamana_graph_with_stats(
        shard_len,
        medoid,
        graph_degree,
        build_list_size,
        alpha,
        shard_seed,
        local_dist,
    );

    // Remap adjacency to global ids. `nodes` is `global_ids` (already
    // ascending); neighbor lists translate local -> global.
    let neighbors = graph
        .neighbors
        .iter()
        .map(|local_neighbors| {
            local_neighbors
                .iter()
                .map(|&local| global_ids[local as usize])
                .collect::<Vec<u32>>()
        })
        .collect();

    ShardGraph {
        nodes: global_ids.to_vec(),
        neighbors,
    }
}

/// Streaming stitch (ADR-085 D8): k-way merge sorted shard cursors by global
/// node id. Cursors retain only headers; adjacency payloads are loaded for the
/// current node group, then discarded after passthrough/prune. The output graph
/// is required result storage and excluded from the incremental D8 bound.
fn stitch_shard_spool<D>(
    node_count: usize,
    shard_spool: &mut ShardSpool,
    graph_degree: usize,
    alpha: f32,
    dist: &D,
) -> Result<(VamanaGraph, ShardBuildStats), String>
where
    D: Fn(u32, u32) -> f32,
{
    let mut graph = VamanaGraph::empty(node_count, graph_degree);
    let spills = std::mem::take(&mut shard_spool.spills);
    let shard_count = spills.len();
    let mut cursors = Vec::with_capacity(shard_count);
    for (shard_index, spill) in spills.into_iter().enumerate() {
        let spill = spill.ok_or_else(|| {
            format!("ec_distann shard {shard_index} was not written to the spool")
        })?;
        cursors.push(ShardCursor::open(spill, node_count, graph_degree)?);
    }
    let mut heap: BinaryHeap<Reverse<(u32, usize)>> = BinaryHeap::with_capacity(shard_count);
    for (shard_index, cursor) in cursors.iter().enumerate() {
        if let Some(header) = cursor.current {
            heap.push(Reverse((header.node, shard_index)));
        }
    }

    let mut edges_before: u64 = 0;
    let mut edges_after: u64 = 0;
    let mut peak_union_len = 0_usize;
    let mut peak_cursor_bytes =
        retained_cursor_bytes(shard_count, cursors.capacity(), heap.capacity())?;
    let mut peak_group_bytes = 0_usize;
    let mut peak_retained_bytes = peak_cursor_bytes;
    let mut expected_node = 0_u32;
    let mut neighbor_scratch: Vec<u32> = Vec::new();

    while let Some(Reverse((node, first_shard))) = heap.pop() {
        maybe_check_for_interrupts();
        if node != expected_node {
            return Err(format!(
                "ec_distann stitched coverage gap: expected node {expected_node}, found {node}"
            ));
        }
        expected_node = expected_node
            .checked_add(1)
            .ok_or_else(|| "ec_distann stitched node id overflow".to_owned())?;

        let mut membership = 0_usize;
        let mut passthrough: Option<Vec<u32>> = None;
        let mut union: Vec<u32> = Vec::new();
        let mut shard = Some(first_shard);
        while let Some(shard_index) = shard {
            let entry_node = cursors[shard_index].consume_into(
                node_count,
                graph_degree,
                &mut neighbor_scratch,
            )?;
            if entry_node != node {
                return Err(format!(
                    "ec_distann shard cursor changed node: expected {node}, got {entry_node}"
                ));
            }
            neighbor_scratch.retain(|&neighbor| neighbor != node);
            let incoming_neighbor_bytes = neighbor_scratch
                .capacity()
                .checked_mul(size_of::<u32>())
                .ok_or_else(|| "ec_distann incoming neighbor accounting overflow".to_owned())?;
            membership += 1;
            let mut prior_union_bytes = 0_usize;
            if membership == 1 {
                passthrough = Some(std::mem::take(&mut neighbor_scratch));
            } else {
                if membership == 2 {
                    union = passthrough.take().unwrap_or_default();
                }
                prior_union_bytes =
                    union
                        .capacity()
                        .checked_mul(size_of::<u32>())
                        .ok_or_else(|| {
                            "ec_distann prior stitch union accounting overflow".to_owned()
                        })?;
                union.extend_from_slice(&neighbor_scratch);
            }

            if let Some(next) = cursors[shard_index].current {
                heap.push(Reverse((next.node, shard_index)));
            }
            let current_cursor_bytes =
                retained_cursor_bytes(shard_count, cursors.capacity(), heap.capacity())?;
            peak_cursor_bytes = peak_cursor_bytes.max(current_cursor_bytes);
            let passthrough_bytes = passthrough.as_ref().map_or(Ok(0), |neighbors| {
                neighbors
                    .capacity()
                    .checked_mul(size_of::<u32>())
                    .ok_or_else(|| "ec_distann passthrough neighbor accounting overflow".to_owned())
            })?;
            let union_bytes = union
                .capacity()
                .checked_mul(size_of::<u32>())
                .ok_or_else(|| "ec_distann stitch union accounting overflow".to_owned())?;
            let retained_group_bytes = passthrough_bytes
                .checked_add(union_bytes)
                .ok_or_else(|| "ec_distann retained group accounting overflow".to_owned())?;
            // For a multi-membership node, retain and account the reusable
            // cursor-read scratch alongside the enlarged union. Conservatively
            // include the prior union allocation as well because `Vec` growth
            // may allocate the replacement before releasing the old buffer.
            // Reusing the read scratch avoids a fresh payload allocation for
            // every membership after the first.
            let active_group_bytes = if membership > 1 {
                retained_group_bytes
                    .checked_add(incoming_neighbor_bytes)
                    .and_then(|bytes| bytes.checked_add(prior_union_bytes))
                    .ok_or_else(|| {
                        "ec_distann active stitch group accounting overflow".to_owned()
                    })?
            } else {
                retained_group_bytes
            };
            peak_group_bytes = peak_group_bytes.max(active_group_bytes);
            let current_retained_bytes = current_cursor_bytes
                .checked_add(active_group_bytes)
                .ok_or_else(|| "ec_distann retained stitch accounting overflow".to_owned())?;
            peak_retained_bytes = peak_retained_bytes.max(current_retained_bytes);

            shard = match heap.peek() {
                Some(Reverse((next_node, _))) if *next_node == node => {
                    let Reverse((_, next_shard)) = heap.pop().expect("peeked heap entry");
                    Some(next_shard)
                }
                _ => None,
            };
        }

        let final_neighbors = if membership == 1 {
            let list = passthrough.unwrap_or_default();
            edges_before += list.len() as u64;
            list
        } else {
            union.sort_unstable();
            union.dedup();
            peak_union_len = peak_union_len.max(union.len());
            edges_before += union.len() as u64;
            let mut candidates: Vec<Candidate> = Vec::with_capacity(union.len());
            candidates.extend(union.iter().map(|&neighbor| Candidate {
                node: neighbor,
                distance: dist(node, neighbor),
            }));
            let union_bytes = union
                .capacity()
                .checked_mul(size_of::<u32>())
                .ok_or_else(|| "ec_distann stitch union accounting overflow".to_owned())?;
            let candidate_bytes = candidates
                .capacity()
                .checked_mul(size_of::<Candidate>())
                .ok_or_else(|| "ec_distann stitch candidate accounting overflow".to_owned())?;
            let result_bytes = graph_degree
                .checked_mul(size_of::<u32>())
                .ok_or_else(|| "ec_distann stitch prune-result accounting overflow".to_owned())?;
            let scratch_bytes = neighbor_scratch
                .capacity()
                .checked_mul(size_of::<u32>())
                .ok_or_else(|| "ec_distann stitch scratch accounting overflow".to_owned())?;
            let prune_bytes = union_bytes
                .checked_add(candidate_bytes)
                .and_then(|bytes| bytes.checked_add(result_bytes))
                .and_then(|bytes| bytes.checked_add(scratch_bytes))
                .ok_or_else(|| "ec_distann stitch prune accounting overflow".to_owned())?;
            let current_cursor_bytes =
                retained_cursor_bytes(shard_count, cursors.capacity(), heap.capacity())?;
            peak_group_bytes = peak_group_bytes.max(prune_bytes);
            let current_retained_bytes = current_cursor_bytes
                .checked_add(prune_bytes)
                .ok_or_else(|| "ec_distann retained prune accounting overflow".to_owned())?;
            peak_retained_bytes = peak_retained_bytes.max(current_retained_bytes);
            robust_prune(node, candidates, alpha, graph_degree, dist)
        };
        edges_after += final_neighbors.len() as u64;
        graph.neighbors[node as usize] = final_neighbors;
    }

    if expected_node as usize != node_count {
        return Err(format!(
            "ec_distann stitched coverage ended at {expected_node}, expected {node_count} nodes"
        ));
    }
    for (shard_index, cursor) in cursors.iter_mut().enumerate() {
        if cursor.remaining_entries != 0 || cursor.current.is_some() {
            return Err(format!(
                "ec_distann shard cursor {shard_index} retained unconsumed entries"
            ));
        }
        cursor
            .io
            .ensure_eof()
            .map_err(|error| format!("ec_distann shard cursor {shard_index}: {error}"))?;
    }

    let stats = ShardBuildStats {
        shard_count,
        duplication_factor: 1.0,
        max_shard_size: 0,
        stitch_edges_before_prune: edges_before,
        stitch_edges_after_prune: edges_after,
        stitch_peak_union_len: peak_union_len,
        shard_output_spill_bytes: shard_spool.bytes_written,
        stitch_peak_cursor_bytes: peak_cursor_bytes,
        stitch_peak_group_bytes: peak_group_bytes,
        stitch_peak_retained_bytes: peak_retained_bytes,
        reachability_repairs: 0,
    };
    Ok((graph, stats))
}

#[cfg(test)]
fn stitch_shard_graphs<D>(
    node_count: usize,
    shard_graphs: Vec<ShardGraph>,
    graph_degree: usize,
    alpha: f32,
    dist: &D,
) -> Result<(VamanaGraph, ShardBuildStats), String>
where
    D: Fn(u32, u32) -> f32,
{
    let mut spool = ShardSpool::create(shard_graphs.len())?;
    for (shard_index, graph) in shard_graphs.into_iter().enumerate() {
        spool.append_shard(shard_index, graph, graph_degree)?;
    }
    stitch_shard_spool(node_count, &mut spool, graph_degree, alpha, dist)
}

/// Guarantee FR-077-CON-3: every node reachable from `medoid`, while never
/// violating FR-077-CON-1 (out-degree ≤ `graph_degree`). Runs BFS over
/// out-edges; each stranded node gets one in-edge from the nearest already-
/// reached node that can accept the edge **without exceeding R** — a node with
/// a free slot (append) or with an evictable non-protected neighbor (replace
/// its farthest such neighbor). Repair edges are protected so a later repair
/// on the same source never re-strands an earlier node (monotone convergence),
/// and reachability is propagated from each repaired node so no node is
/// repaired that is already navigable through a repaired predecessor.
///
/// Such an accepting source always exists for `graph_degree >= 2`: the number
/// of protected repair edges never exceeds the number of reached nodes (each
/// repair marks ≥1 new node reached), which is `< reached_count * graph_degree`
/// total slots, so some reached node always has a free or non-protected slot.
/// `graph_degree` is reloption-bounded to ≥ 4 (`ECDISTANN_MIN_GRAPH_DEGREE`),
/// so the "no accepting source" branch is unreachable in practice; it is an
/// `Err` rather than a silent CON-1 violation. Deterministic (stranded nodes
/// processed ascending). Returns the repair count (expected 0 at corpus scale).
fn repair_reachability<D>(
    graph: &mut VamanaGraph,
    medoid: u32,
    graph_degree: usize,
    dist: &D,
) -> Result<usize, String>
where
    D: Fn(u32, u32) -> f32,
{
    let node_count = graph.node_count();
    if node_count == 0 {
        return Ok(0);
    }
    let reached_order = bfs_reachable(graph, medoid);
    if reached_order.len() == node_count {
        return Ok(0);
    }

    let mut reached = vec![false; node_count];
    for &n in &reached_order {
        reached[n as usize] = true;
    }
    let mut reached_list: Vec<u32> = reached_order;
    // Edges added by the repair must never be evicted, else a later repair on
    // the same source could re-strand an earlier node.
    let mut protected: Vec<Vec<u32>> = vec![Vec::new(); node_count];

    let mut repairs = 0_usize;
    for node in 0..node_count as u32 {
        maybe_check_for_interrupts();
        if reached[node as usize] {
            continue;
        }
        // With reachability propagation, an unreached node has no in-edge from
        // any reached node, so a fresh in-edge is always needed. Pick the
        // nearest reached source that can accept it without exceeding R: it has
        // a free slot, or a non-protected neighbor we can evict.
        let mut best_src: Option<(u32, f32, bool)> = None; // (src, dist, has_room)
        for &candidate in &reached_list {
            let adjacency = &graph.neighbors[candidate as usize];
            let has_room = adjacency.len() < graph_degree;
            let has_evictable = !has_room
                && adjacency
                    .iter()
                    .any(|n| !protected[candidate as usize].contains(n));
            if has_room || has_evictable {
                let d = dist(candidate, node);
                if best_src.map_or(true, |(_, bd, _)| d < bd) {
                    best_src = Some((candidate, d, has_room));
                }
            }
        }

        let (src, has_room) = match best_src {
            Some((src, _, has_room)) => (src, has_room),
            // Unreachable for graph_degree >= 2 (see fn docs); never exceed R.
            None => {
                return Err(format!(
                    "ec_distann reachability repair could not place an in-edge to node {node} \
                     without exceeding graph_degree {graph_degree} (degenerate saturation)"
                ))
            }
        };

        if has_room {
            graph.neighbors[src as usize].push(node);
        } else {
            // Evict the farthest non-protected neighbor (exists: has_evictable).
            let protected_src = &protected[src as usize];
            let victim = graph.neighbors[src as usize]
                .iter()
                .enumerate()
                .filter(|(_, n)| !protected_src.contains(n))
                .map(|(i, &n)| (i, dist(src, n)))
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
                .expect("has_evictable guarantees a non-protected neighbor");
            graph.neighbors[src as usize][victim] = node;
        }
        protected[src as usize].push(node);
        repairs += 1;
        debug_assert!(graph.neighbors[src as usize].len() <= graph_degree);

        // Propagate reachability from the newly-connected node: everything it
        // can now reach is reached too, so no node navigable through a repaired
        // predecessor is repaired again.
        let mut stack = vec![node];
        reached[node as usize] = true;
        reached_list.push(node);
        while let Some(cur) = stack.pop() {
            for &neighbor in &graph.neighbors[cur as usize] {
                if !reached[neighbor as usize] {
                    reached[neighbor as usize] = true;
                    reached_list.push(neighbor);
                    stack.push(neighbor);
                }
            }
        }
    }

    Ok(repairs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    /// Deterministic unit-normalized random corpus for the property suite.
    fn random_corpus(node_count: usize, dimensions: usize, seed: u64) -> Vec<Vec<f32>> {
        let mut rng = StdRng::seed_from_u64(seed);
        (0..node_count)
            .map(|_| {
                let mut v: Vec<f32> = (0..dimensions)
                    .map(|_| rng.gen_range(-1.0_f32..1.0))
                    .collect();
                let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-6);
                v.iter_mut().for_each(|x| *x /= norm);
                v
            })
            .collect()
    }

    fn refs(corpus: &[Vec<f32>]) -> Vec<&[f32]> {
        corpus.iter().map(Vec::as_slice).collect()
    }

    fn build(
        corpus: &[Vec<f32>],
        dimensions: usize,
        graph_degree: usize,
        shard_count: usize,
        closure_epsilon: f32,
        seed: u64,
    ) -> (VamanaGraph, u32, ShardBuildStats) {
        let refs = refs(corpus);
        build_sharded_graph(
            &refs,
            dimensions,
            graph_degree,
            /* build_list_size */ 64,
            /* alpha */ 1.2,
            shard_count,
            closure_epsilon,
            seed,
        )
        .expect("sharded build should succeed")
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        // FR-077-CON-1: post-stitch out-degree <= graph_degree for every node.
        #[test]
        fn tc038_degree_bounded(
            node_count in 40_usize..160,
            dimensions in 4_usize..12,
            graph_degree in 4_usize..24,
            shard_count in 2_usize..6,
            eps in 0.0_f32..0.4,
            seed in any::<u64>(),
        ) {
            let corpus = random_corpus(node_count, dimensions, seed);
            let (graph, _medoid, _stats) =
                build(&corpus, dimensions, graph_degree, shard_count, eps, seed);
            for node in 0..node_count {
                prop_assert!(
                    graph.neighbors[node].len() <= graph_degree,
                    "node {} degree {} exceeds R {}",
                    node,
                    graph.neighbors[node].len(),
                    graph_degree
                );
            }
        }

        // FR-077-CON-2: every vec_id appears exactly once (the graph has
        // exactly node_count adjacency slots, each a valid distinct node id),
        // and no self-loops or dangling neighbor ids.
        #[test]
        fn tc038_uniqueness_and_valid_edges(
            node_count in 40_usize..160,
            dimensions in 4_usize..12,
            graph_degree in 4_usize..24,
            shard_count in 2_usize..6,
            eps in 0.0_f32..0.4,
            seed in any::<u64>(),
        ) {
            let corpus = random_corpus(node_count, dimensions, seed);
            let (graph, _medoid, _stats) =
                build(&corpus, dimensions, graph_degree, shard_count, eps, seed);
            prop_assert_eq!(graph.node_count(), node_count);
            for node in 0..node_count {
                let neighbors = &graph.neighbors[node];
                let mut seen = std::collections::HashSet::new();
                for &nb in neighbors {
                    prop_assert!((nb as usize) < node_count, "dangling neighbor {}", nb);
                    prop_assert_ne!(nb as usize, node, "self-loop at {}", node);
                    prop_assert!(seen.insert(nb), "duplicate neighbor {} at {}", nb, node);
                }
            }
        }

        // FR-077-CON-3: every node reachable from the entry medoid.
        #[test]
        fn tc038_medoid_reachability(
            node_count in 40_usize..160,
            dimensions in 4_usize..12,
            graph_degree in 4_usize..24,
            shard_count in 2_usize..6,
            eps in 0.0_f32..0.4,
            seed in any::<u64>(),
        ) {
            let corpus = random_corpus(node_count, dimensions, seed);
            let (graph, medoid, _stats) =
                build(&corpus, dimensions, graph_degree, shard_count, eps, seed);
            let reached = bfs_reachable(&graph, medoid);
            prop_assert_eq!(
                reached.len(),
                node_count,
                "only {} of {} nodes reachable from medoid",
                reached.len(),
                node_count
            );
        }

        // FR-077 determinism: identical corpus + seed + options => identical
        // stitched graph.
        #[test]
        fn tc038_determinism(
            node_count in 40_usize..160,
            dimensions in 4_usize..12,
            graph_degree in 4_usize..24,
            shard_count in 2_usize..6,
            eps in 0.0_f32..0.4,
            seed in any::<u64>(),
        ) {
            let corpus = random_corpus(node_count, dimensions, seed);
            let (graph_a, medoid_a, stats_a) =
                build(&corpus, dimensions, graph_degree, shard_count, eps, seed);
            let (graph_b, medoid_b, stats_b) =
                build(&corpus, dimensions, graph_degree, shard_count, eps, seed);
            prop_assert_eq!(medoid_a, medoid_b);
            prop_assert_eq!(stats_a, stats_b);
            prop_assert_eq!(graph_a.neighbors, graph_b.neighbors);
        }

        // FR-077-CON (alpha-prune invariant, TC-038 / spec/tests.md): the
        // stitched *union* preserves robust_prune's alpha-diversity. For every
        // multi-shard node (the nodes the stitch actually re-prunes over their
        // unioned edge lists), the kept neighbor list satisfies, in kept order,
        // that no earlier-kept (closer) neighbor alpha-dominates a later one
        // under the GLOBAL distance: alpha * dist(nb[i], nb[j]) > dist(node,
        // nb[j]) for i < j. Single-shard passthrough nodes carry the per-shard
        // Vamana output (backlink re-prunes reorder that list, so it is not a
        // single-prune kept order) and are exempt; the reachability repair runs
        // after this, so it is checked on the pre-repair stitch output.
        #[test]
        fn tc038_alpha_prune_invariant(
            node_count in 40_usize..160,
            dimensions in 4_usize..12,
            graph_degree in 4_usize..24,
            shard_count in 2_usize..6,
            eps in 0.0_f32..0.4,
            seed in any::<u64>(),
        ) {
            let alpha = 1.2_f32;
            let corpus = random_corpus(node_count, dimensions, seed);
            let refs = refs(&corpus);
            let dist = |a: u32, b: u32| {
                source_inner_product_distance(refs[a as usize], refs[b as usize])
            };
            let assignment =
                assign_shards(&refs, dimensions, shard_count, eps, seed).unwrap();
            // Membership count per global node: the stitch re-prunes exactly the
            // nodes appearing in >= 2 shards.
            let mut membership = vec![0_u32; node_count];
            for shard in &assignment.members {
                for &n in shard {
                    membership[n as usize] += 1;
                }
            }
            let shard_graphs =
                build_shard_graphs(&assignment.members, graph_degree, 64, alpha, seed, &dist);
            let (graph, _stats) = stitch_shard_graphs(
                node_count,
                shard_graphs,
                graph_degree,
                alpha,
                &dist,
            )
            .unwrap();

            let mut checked_multi = 0_usize;
            for node in 0..node_count as u32 {
                if membership[node as usize] < 2 {
                    continue; // passthrough node — exempt (see comment above)
                }
                checked_multi += 1;
                let neighbors = &graph.neighbors[node as usize];
                for j in 0..neighbors.len() {
                    let vj = neighbors[j];
                    let d_node_vj = dist(node, vj);
                    for &vi in neighbors.iter().take(j) {
                        // vi kept before vj => vi must not alpha-dominate vj.
                        let lhs = alpha * dist(vi, vj);
                        prop_assert!(
                            lhs + 1e-4 >= d_node_vj,
                            "alpha-prune violated at node {}: alpha*d({},{})={} < d(node,{})={}",
                            node, vi, vj, lhs, vj, d_node_vj
                        );
                    }
                }
            }
            // `checked_multi` is diagnostic only: some well-separated random
            // corpora legitimately produce no closure overlap even at eps>0,
            // in which case the invariant holds vacuously. Closure duplication
            // is exercised on average by tc038_closure_duplication_scales.
            let _ = checked_multi;
        }
    }

    // FR-077-AC-2: stitching an already-stitched graph is a no-op. Model the
    // stitched graph as a single shard covering all nodes and re-stitch: every
    // node is single-membership, so the passthrough path must reproduce it
    // exactly.
    #[test]
    fn tc038_stitch_idempotence() {
        let dimensions = 8;
        let node_count = 120;
        let graph_degree = 16;
        let corpus = random_corpus(node_count, dimensions, 0xABCD);
        let refs = refs(&corpus);
        let dist =
            |a: u32, b: u32| source_inner_product_distance(refs[a as usize], refs[b as usize]);

        let (graph, _medoid, _stats) = build(&corpus, dimensions, graph_degree, 4, 0.2, 0xABCD);

        // Present the stitched graph as one shard covering every node.
        let single = ShardGraph {
            nodes: (0..node_count as u32).collect(),
            neighbors: graph.neighbors.clone(),
        };
        let (restitched, _stats) =
            stitch_shard_graphs(node_count, vec![single], graph_degree, 1.2, &dist)
                .expect("single-shard spool stitch should succeed");
        assert_eq!(
            restitched.neighbors, graph.neighbors,
            "re-stitching a single-shard graph must be a no-op"
        );
    }

    // FR-077-CON-4 / ADR-085 D8: shard outputs are represented by spill bytes,
    // while the stitch retains only fixed cursor headers plus one node-group
    // union/prune workspace. This is an allocation-accounting assertion over
    // the exact runtime merge; the pg_test build exercises the BufFile-backed
    // I/O implementation rather than this unit-test memory transport.
    #[test]
    fn tc038_d8_spill_and_cursor_bound() {
        let dimensions = 12;
        let node_count = 800;
        let graph_degree = 16;
        let shard_count = 8;
        let corpus = random_corpus(node_count, dimensions, 0xD8);
        let (_graph, _medoid, stats) =
            build(&corpus, dimensions, graph_degree, shard_count, 0.3, 0xD8);

        assert!(stats.shard_output_spill_bytes > 0);
        assert!(stats.stitch_peak_cursor_bytes >= pg_sys::BLCKSZ as usize);
        assert!(stats.stitch_peak_group_bytes > 0);
        assert!(
            stats.stitch_peak_retained_bytes
                <= stats.stitch_peak_cursor_bytes + stats.stitch_peak_group_bytes
        );

        // Vec capacity growth is at most the next power-of-two (<2x logical
        // entries). The conservative accounting includes current + prior
        // union allocations during growth, an R-sized cursor scratch, one
        // Candidate per unique id, and an R-sized prune result.
        let max_group_nodes = shard_count * graph_degree;
        let conservative_group_bound = 4 * max_group_nodes * size_of::<u32>()
            + 2 * max_group_nodes * size_of::<Candidate>()
            + 3 * graph_degree * size_of::<u32>();
        assert!(
            stats.stitch_peak_group_bytes <= conservative_group_bound,
            "group bytes {} exceed conservative one-group bound {}",
            stats.stitch_peak_group_bytes,
            conservative_group_bound
        );
        assert!(
            stats.stitch_peak_retained_bytes < stats.shard_output_spill_bytes as usize,
            "stitch retained {} bytes but spool contains only {} bytes",
            stats.stitch_peak_retained_bytes,
            stats.shard_output_spill_bytes
        );
    }

    fn raw_spool(shards: Vec<(usize, Vec<u8>)>) -> ShardSpool {
        let bytes_written = shards.iter().map(|(_, bytes)| bytes.len() as u64).sum();
        ShardSpool {
            spills: shards
                .into_iter()
                .map(|(entry_count, bytes)| {
                    Some(ShardSpill {
                        io: ShardSpoolIo { bytes, position: 0 },
                        entry_count,
                    })
                })
                .collect(),
            bytes_written,
        }
    }

    fn raw_entry(node: u32, neighbors: &[u32]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(
            DISTANN_SHARD_SPILL_HEADER_BYTES + neighbors.len() * size_of::<u32>(),
        );
        bytes.extend_from_slice(&node.to_le_bytes());
        bytes.extend_from_slice(&(neighbors.len() as u32).to_le_bytes());
        for &neighbor in neighbors {
            bytes.extend_from_slice(&neighbor.to_le_bytes());
        }
        bytes
    }

    fn corrupt_spool_error(
        node_count: usize,
        graph_degree: usize,
        spool: &mut ShardSpool,
    ) -> String {
        stitch_shard_spool(node_count, spool, graph_degree, 1.2, &|left, right| {
            left.abs_diff(right) as f32
        })
        .expect_err("corrupt shard spool must fail closed")
    }

    #[test]
    fn tc038_corrupt_spool_rejects_coverage_gap() {
        let mut bytes = raw_entry(0, &[2]);
        bytes.extend_from_slice(&raw_entry(2, &[0]));
        let error = corrupt_spool_error(3, 4, &mut raw_spool(vec![(2, bytes)]));
        assert!(
            error.contains("coverage gap: expected node 1, found 2"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn tc038_corrupt_spool_rejects_out_of_range_node_and_neighbor() {
        let node_error = corrupt_spool_error(2, 4, &mut raw_spool(vec![(1, raw_entry(2, &[]))]));
        assert!(
            node_error.contains("node 2 outside node count 2"),
            "unexpected node error: {node_error}"
        );

        let neighbor_error =
            corrupt_spool_error(2, 4, &mut raw_spool(vec![(1, raw_entry(0, &[2]))]));
        assert!(
            neighbor_error.contains("neighbor 2 outside node count 2"),
            "unexpected neighbor error: {neighbor_error}"
        );
    }

    #[test]
    fn tc038_corrupt_spool_rejects_over_degree_entry() {
        let error = corrupt_spool_error(3, 1, &mut raw_spool(vec![(1, raw_entry(0, &[1, 2]))]));
        assert!(
            error.contains("node 0 has 2 neighbors, exceeding R=1"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn tc038_corrupt_spool_rejects_unsorted_stream() {
        let mut bytes = raw_entry(0, &[1]);
        bytes.extend_from_slice(&raw_entry(0, &[1]));
        let error = corrupt_spool_error(2, 4, &mut raw_spool(vec![(2, bytes)]));
        assert!(
            error.contains("not strictly sorted: 0 after 0"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn tc038_corrupt_spool_rejects_unconsumed_tail() {
        let mut bytes = raw_entry(0, &[]);
        bytes.push(0xA5);
        let error = corrupt_spool_error(1, 4, &mut raw_spool(vec![(1, bytes)]));
        assert!(
            error.contains("unconsumed trailing bytes"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn tc038_corrupt_spool_rejects_truncated_payload() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        let error = corrupt_spool_error(2, 4, &mut raw_spool(vec![(1, bytes)]));
        assert!(
            error.contains("truncated test shard spool entry"),
            "unexpected error: {error}"
        );
    }

    // Closure overlap actually duplicates nodes when epsilon > 0, and never
    // when epsilon == 0 (each node lands in exactly one shard).
    #[test]
    fn tc038_closure_duplication_scales_with_epsilon() {
        let dimensions = 12;
        let node_count = 300;
        let corpus = random_corpus(node_count, dimensions, 7);

        let (_g0, _m0, stats0) = build(&corpus, dimensions, 16, 6, 0.0, 7);
        assert!(
            (stats0.duplication_factor - 1.0).abs() < 1e-9,
            "epsilon=0 must not duplicate, got {}",
            stats0.duplication_factor
        );

        let (_g1, _m1, stats1) = build(&corpus, dimensions, 16, 6, 0.3, 7);
        assert!(
            stats1.duplication_factor > 1.0,
            "epsilon=0.3 should duplicate some boundary nodes, got {}",
            stats1.duplication_factor
        );
    }

    // resolve_shard_count honors explicit requests and the monolithic default.
    #[test]
    fn resolve_shard_count_behaviour() {
        assert_eq!(
            resolve_shard_count(10_000, 0),
            1,
            "small corpus auto = monolithic"
        );
        assert_eq!(resolve_shard_count(100_000, 0), 4, "100k auto shards");
        assert_eq!(resolve_shard_count(5_000, 8), 8, "explicit request honored");
        assert_eq!(
            resolve_shard_count(3, 8),
            3,
            "request clamped to node_count"
        );
        assert_eq!(resolve_shard_count(1_000_000, 0), 16, "auto shard cap");
    }

    // FR-077-CON-1 + CON-3 regression: repair_reachability must connect a fully
    // disconnected component from the medoid WITHOUT any node exceeding
    // graph_degree, even when every reachable source already sits at R (the
    // degenerate saturation the reviewer flagged: the old code appended past R
    // and would have failed staging). Two triangles at R=2, medoid in one.
    #[test]
    fn repair_reachability_bounds_degree_on_disconnected_graph() {
        let graph_degree = 2;
        let node_count = 6;
        let mut graph = VamanaGraph::empty(node_count, graph_degree);
        // Component A {0,1,2} (contains medoid 0), component B {3,4,5}.
        graph.neighbors[0] = vec![1, 2];
        graph.neighbors[1] = vec![0, 2];
        graph.neighbors[2] = vec![0, 1];
        graph.neighbors[3] = vec![4, 5];
        graph.neighbors[4] = vec![3, 5];
        graph.neighbors[5] = vec![3, 4];
        // Every source in component A is already at R=2, so the repair must
        // evict a non-protected neighbor rather than append past R.
        let dist = |a: u32, b: u32| (a as f32 - b as f32).abs();
        let repairs = repair_reachability(&mut graph, 0, graph_degree, &dist)
            .expect("repair must succeed without exceeding R");
        assert!(
            repairs >= 1,
            "component B was stranded and must be repaired"
        );

        // CON-1: no node exceeds graph_degree.
        for (node, neighbors) in graph.neighbors.iter().enumerate() {
            assert!(
                neighbors.len() <= graph_degree,
                "node {node} degree {} exceeds R {graph_degree}",
                neighbors.len()
            );
        }
        // CON-3: every node reachable from the medoid.
        let reached = bfs_reachable(&graph, 0);
        assert_eq!(
            reached.len(),
            node_count,
            "all nodes must be reachable post-repair"
        );
    }
}
