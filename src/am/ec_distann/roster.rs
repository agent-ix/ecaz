//! FR-078/FR-082 roster + epoch configuration for the M2 two-node read path
//! (Task 164).
//!
//! In M2 the node roster and active epoch are operator-configured GUCs; M3
//! (FR-082 full) replaces this with the persisted epoch manifest / publish
//! hand-off. The three GUCs are:
//!
//! - `ec_distann.roster` — placement-ordered node list, one entry per node as
//!   `node_id@conninfo`, entries separated by `;`. The `conninfo` is a libpq
//!   connection string (may contain spaces and `=`, but not `@` or `;`). An
//!   empty roster is the single-node degenerate case (the coordinator owns
//!   every vec_id, no remote calls).
//! - `ec_distann.local_node_id` — this instance's node id; the roster entry
//!   with this id is the local node (expanded in-process, no round trip).
//! - `ec_distann.epoch` — the active epoch number, part of the FR-082
//!   fingerprint.
//!
//! The parser (`parse_roster`) is pure and unit-tested; the GUC plumbing and
//! the `DistannPlacementDirectory` / `DistannEpochIdentity` builders sit on top
//! and are exercised by the M2 pg-tests + the two-node fixture.

// The directory/identity builders are consumed by the expand endpoint + remote
// expander in the following M2 slices. Allow dead_code until they land.
#![allow(dead_code)]

use std::ffi::CString;

use pgrx::{GucContext, GucFlags, GucRegistry, GucSetting};

use super::epoch::DistannEpochIdentity;
use super::page::DistannMetadataPage;
use super::placement::{
    DistannPlacementDirectory, DistannRosterNode, DISTANN_PLACEMENT_HASH_V1,
};

static ECDISTANN_ROSTER_GUC: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(None);
static ECDISTANN_LOCAL_NODE_ID_GUC: GucSetting<i32> = GucSetting::<i32>::new(0);
/// PostgreSQL int GUCs are 32-bit; epoch numbers at research scale fit easily.
static ECDISTANN_EPOCH_GUC: GucSetting<i32> = GucSetting::<i32>::new(0);

pub(super) fn register_gucs() {
    GucRegistry::define_string_guc(
        c"ec_distann.roster",
        c"FR-078 placement-ordered node roster for multi-node ec_distann scans.",
        c"Entries `node_id@conninfo` separated by `;`, in placement order (owning-node index order). Empty = single-node (the coordinator owns every vec_id). M2 operator config; M3 replaces it with the persisted epoch manifest.",
        &ECDISTANN_ROSTER_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_distann.local_node_id",
        c"This instance's node id within ec_distann.roster.",
        c"The roster entry whose node_id equals this value is the local node (expanded in-process, no round trip). Ignored when the roster is empty.",
        &ECDISTANN_LOCAL_NODE_ID_GUC,
        0,
        i32::MAX,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_distann.epoch",
        c"Active ec_distann epoch number (FR-082 fingerprint input).",
        c"Part of the epoch fingerprint every ec_distann_expand_nodes call validates. M2 operator config; M3 sources it from the published epoch manifest.",
        &ECDISTANN_EPOCH_GUC,
        0,
        i32::MAX,
        GucContext::Userset,
        GucFlags::default(),
    );
}

pub(super) fn current_local_node_id() -> u32 {
    u32::try_from(ECDISTANN_LOCAL_NODE_ID_GUC.get().max(0)).unwrap_or(0)
}

pub(super) fn current_epoch() -> u64 {
    u64::try_from(ECDISTANN_EPOCH_GUC.get().max(0)).unwrap_or(0)
}

pub(super) fn current_roster_spec() -> String {
    ECDISTANN_ROSTER_GUC
        .get()
        .map(|cstring| cstring.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// One parsed roster entry: `(node_id, conninfo)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParsedRosterEntry {
    pub(super) node_id: u32,
    pub(super) conninfo: String,
}

/// Parse the `ec_distann.roster` GUC spec into placement-ordered entries.
/// Pure: the whole risk surface of the config lives here. Empty/whitespace
/// spec yields an empty roster (single-node degenerate case).
pub(super) fn parse_roster(spec: &str) -> Result<Vec<ParsedRosterEntry>, String> {
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();
    for (index, raw) in trimmed.split(';').enumerate() {
        let entry = raw.trim();
        if entry.is_empty() {
            // A trailing `;` or blank segment is tolerated.
            continue;
        }
        let (id_str, conninfo) = entry.split_once('@').ok_or_else(|| {
            format!(
                "ec_distann.roster entry {index} ({entry:?}) must be `node_id@conninfo`"
            )
        })?;
        let node_id: u32 = id_str.trim().parse().map_err(|_| {
            format!("ec_distann.roster entry {index} has non-numeric node_id {id_str:?}")
        })?;
        let conninfo = conninfo.trim();
        if conninfo.is_empty() {
            return Err(format!(
                "ec_distann.roster entry {index} (node {node_id}) has an empty conninfo"
            ));
        }
        if !seen_ids.insert(node_id) {
            return Err(format!(
                "ec_distann.roster has a duplicate node_id {node_id}"
            ));
        }
        entries.push(ParsedRosterEntry {
            node_id,
            conninfo: conninfo.to_owned(),
        });
    }
    Ok(entries)
}

/// Build the active `DistannPlacementDirectory` from the roster + epoch GUCs.
/// An empty roster produces a single-node directory owning everything locally.
pub(super) fn current_placement_directory() -> Result<DistannPlacementDirectory, String> {
    let entries = parse_roster(&current_roster_spec())?;
    let epoch = current_epoch();
    if entries.is_empty() {
        // Single-node degenerate: one local node, owns every vec_id.
        return Ok(DistannPlacementDirectory {
            epoch,
            hash_version: DISTANN_PLACEMENT_HASH_V1,
            nodes: vec![DistannRosterNode {
                node_id: current_local_node_id(),
                is_local: true,
                conninfo: String::new(),
            }],
            record_counts: Vec::new(),
        });
    }
    let local_node_id = current_local_node_id();
    let mut local_seen = false;
    let nodes: Vec<DistannRosterNode> = entries
        .into_iter()
        .map(|entry| {
            let is_local = entry.node_id == local_node_id;
            local_seen |= is_local;
            DistannRosterNode {
                node_id: entry.node_id,
                is_local,
                conninfo: if is_local { String::new() } else { entry.conninfo },
            }
        })
        .collect();
    if !local_seen {
        return Err(format!(
            "ec_distann.local_node_id {local_node_id} is not present in ec_distann.roster"
        ));
    }
    Ok(DistannPlacementDirectory {
        epoch,
        hash_version: DISTANN_PLACEMENT_HASH_V1,
        nodes,
        record_counts: Vec::new(),
    })
}

/// Derive the local epoch identity (for fingerprint compute/compare) from the
/// active roster directory + this index's build-time metadata.
pub(super) fn local_epoch_identity(
    directory: &DistannPlacementDirectory,
    metadata: &DistannMetadataPage,
) -> DistannEpochIdentity {
    DistannEpochIdentity {
        epoch: directory.epoch,
        format_version: metadata.format_version,
        placement_hash_version: directory.hash_version,
        roster_node_ids: directory.nodes.iter().map(|n| n.node_id).collect(),
        node_count: metadata.node_count,
        dimensions: metadata.dimensions,
        graph_degree: metadata.graph_degree_r,
        neighbor_codec_kind: metadata.neighbor_codec_kind,
        build_seed: metadata.seed,
        content_digest: metadata.content_digest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_roster_empty_is_single_node() {
        assert_eq!(parse_roster("").unwrap(), Vec::new());
        assert_eq!(parse_roster("   ").unwrap(), Vec::new());
    }

    #[test]
    fn parse_roster_two_nodes() {
        let spec = "10@host=/tmp port=28818 dbname=b;20@host=/tmp port=28819 dbname=b";
        let entries = parse_roster(spec).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].node_id, 10);
        assert_eq!(entries[0].conninfo, "host=/tmp port=28818 dbname=b");
        assert_eq!(entries[1].node_id, 20);
        assert_eq!(entries[1].conninfo, "host=/tmp port=28819 dbname=b");
    }

    #[test]
    fn parse_roster_tolerates_trailing_separator_and_whitespace() {
        let spec = " 1@conninfo-a ; 2@conninfo-b ; ";
        let entries = parse_roster(spec).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1].conninfo, "conninfo-b");
    }

    #[test]
    fn parse_roster_rejects_malformed_entries() {
        assert!(parse_roster("noatsign").is_err(), "missing @");
        assert!(parse_roster("x@conninfo").is_err(), "non-numeric id");
        assert!(parse_roster("1@").is_err(), "empty conninfo");
        assert!(parse_roster("1@a;1@b").is_err(), "duplicate id");
    }

    #[test]
    fn placement_order_matches_spec_order() {
        // Ordering is significant: owning_node indexes into it. Verify parse
        // preserves the given order (not sorted by id).
        let entries = parse_roster("30@c;10@a;20@b").unwrap();
        assert_eq!(
            entries.iter().map(|e| e.node_id).collect::<Vec<_>>(),
            vec![30, 10, 20]
        );
    }
}
