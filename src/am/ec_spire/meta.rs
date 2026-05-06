//! Root/control metadata, epoch, and placement-map codecs.

use std::collections::HashMap;

use super::assign::{SPIRE_FIRST_LOCAL_VEC_SEQ, SPIRE_FIRST_PID};
use crate::storage::page::{ItemPointer, ITEM_POINTER_BYTES};

pub(super) const SPIRE_LOCAL_NODE_ID: u32 = 0;
pub(super) const SPIRE_SINGLE_LOCAL_STORE_ID: u32 = 0;
pub(super) const SPIRE_DEFAULT_LOCAL_STORE_GENERATION: u64 = 0;
pub(super) const SPIRE_MIN_EPOCH_RETENTION_SECS: u32 = 10 * 60;
pub(super) const SPIRE_FAILED_EPOCH_RETENTION_SECS: u32 = 60 * 60;
pub(super) const SPIRE_MAX_RETAINED_RETIRED_EPOCHS: u16 = 2;

const META_FORMAT_VERSION: u16 = 1;
const ROOT_CONTROL_MAGIC: u32 = 0x4352_5345; // "ESRC" as little-endian bytes.
const ROOT_CONTROL_STATE_BYTES: usize = 4 + 2 + 2 + 8 + 8 + 8 + ITEM_POINTER_BYTES * 3;
const LOCAL_STORE_CONFIG_MAGIC: u32 = 0x534c_5345; // "ESLS" as little-endian bytes.
const LOCAL_STORE_CONFIG_HEADER_BYTES: usize = 4 + 2 + 2 + 8 + 4;
const LOCAL_STORE_DESCRIPTOR_BYTES: usize = 4 + 4 + 4 + 1 + 3;
const PLACEMENT_DIRECTORY_MAGIC: u32 = 0x4450_5345; // "ESPD" as little-endian bytes.
const PLACEMENT_DIRECTORY_HEADER_BYTES: usize = 4 + 2 + 2 + 8 + 4;
const OBJECT_MANIFEST_MAGIC: u32 = 0x4d4f_5345; // "ESOM" as little-endian bytes.
const OBJECT_MANIFEST_HEADER_BYTES: usize = 4 + 2 + 2 + 8 + 4;
const EPOCH_MANIFEST_MAGIC: u32 = 0x454d_5345; // "ESME" as little-endian bytes.
const PLACEMENT_ENTRY_BYTES: usize = 2 + 1 + 1 + 8 + 8 + 4 + 4 + 4 + 8 + ITEM_POINTER_BYTES + 4;
const EPOCH_MANIFEST_BYTES: usize = 4 + 2 + 1 + 1 + 8 + 8 + 8 + 8;
const MANIFEST_ENTRY_BYTES: usize = 2 + 2 + 8 + 8 + 8 + ITEM_POINTER_BYTES;

include!("meta/root_control.rs");
include!("meta/states.rs");
include!("meta/local_store.rs");
include!("meta/placement.rs");
include!("meta/placement_directory.rs");
include!("meta/epoch.rs");
include!("meta/object_manifest.rs");
include!("meta/snapshot.rs");
include!("meta/tests.rs");
