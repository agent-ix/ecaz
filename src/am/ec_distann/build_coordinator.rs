//! Coordinator build registration and durable private participant bindings.

use std::cell::{Cell, RefCell};

use pgrx::datum::Uuid;
use pgrx::iter::TableIterator;
use pgrx::{name, pg_extern, pg_sys, PgRelation, Spi};

use crate::storage::relation::index_heap_relation_oid_handle;

use super::canonical_wire::{domain_digest, is_rfc4122_v4_uuid, CanonicalEncoder};
use super::generation_catalog::extension_relation_name;
use super::generation_descriptor::{
    validate_roster, DistannBuildOptions, DistannBuildSpec, DistannGenerationDescriptor,
    DistannRosterEntry,
};
use super::handoff_router::{DistannHandoffRouteIdentity, DistannStageAck};
use super::lifecycle_wire::DistannBuildCandidateV1;
use super::manifest_v2::{
    DistannEpochManifestV2, DistannManifestBuildOptions, DistannManifestCodecParameters,
    DistannSourceSnapshot,
};
use super::generation_store::{control_compatibility_digest, open_control_index};
use super::roster_digest;
use super::row_schema::resolve_relation_schema;

const BUILD_REGISTRATION_VERSION: u16 = 1;
const BUILD_REGISTRATION_DOMAIN: &[u8] = b"ec_distann_build_registration_v1\0";
const BUILD_ROSTER_SNAPSHOT_VERSION: u16 = 1;

thread_local! {
    static SOURCE_SESSION_LOCKS: RefCell<Vec<SourceSessionLock>> = const { RefCell::new(Vec::new()) };
    static PENDING_SESSION_RELEASES: RefCell<Vec<PendingSessionRelease>> = const { RefCell::new(Vec::new()) };
    static NEXT_SESSION_RELEASE_ID: Cell<u64> = const { Cell::new(1) };
    static SOURCE_SESSION_EXIT_CALLBACK_REGISTERED: Cell<bool> = const { Cell::new(false) };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceSessionLock {
    source_relation_oid: pg_sys::Oid,
    control_index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
    /// `Some` until the transaction that acquired the nontransactional
    /// session lock commits. Subtransaction promotion rewrites this to the
    /// parent subxid; abort releases it.
    pending_subid: Option<pg_sys::SubTransactionId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingSessionRelease {
    id: u64,
    lock: SourceSessionLock,
    pending_subid: pg_sys::SubTransactionId,
}

fn unlock_session_relation(relation_oid: pg_sys::Oid, lockmode: pg_sys::LOCKMODE) {
    let mut lock_relation_id = pg_sys::LockRelId {
        relId: relation_oid,
        dbId: unsafe { pg_sys::MyDatabaseId },
    };
    unsafe { pg_sys::UnlockRelationIdForSession(&mut lock_relation_id, lockmode) };
}

fn unlock_build_session_relations(lock: SourceSessionLock) {
    // Release in reverse acquisition order.
    unlock_session_relation(
        lock.control_index_oid,
        pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
    );
    unlock_session_relation(
        lock.source_relation_oid,
        pg_sys::ShareLock as pg_sys::LOCKMODE,
    );
}

fn same_build_lock(left: &SourceSessionLock, right: &SourceSessionLock) -> bool {
    left.source_relation_oid == right.source_relation_oid
        && left.control_index_oid == right.control_index_oid
        && left.logical_index_uuid == right.logical_index_uuid
        && left.build_id == right.build_id
}

#[pgrx::pg_guard]
unsafe extern "C-unwind" fn source_session_before_shmem_exit(
    _code: std::ffi::c_int,
    _arg: pg_sys::Datum,
) {
    SOURCE_SESSION_LOCKS.with(|locks| {
        for lock in locks.borrow_mut().drain(..) {
            unlock_build_session_relations(lock);
        }
    });
}

#[pgrx::pg_guard]
unsafe extern "C-unwind" fn source_session_top_xact_abort(
    event: pg_sys::XactEvent::Type,
    _arg: *mut std::ffi::c_void,
) {
    if matches!(
        event,
        pg_sys::XactEvent::XACT_EVENT_ABORT | pg_sys::XactEvent::XACT_EVENT_PARALLEL_ABORT
    ) {
        // ProcReleaseLocks(false) runs after Xact callbacks and removes every
        // DEFAULT_LOCKMETHOD session lock, including ownership committed by
        // an earlier transaction. Clear the backend-local mirror now; the
        // durable registration remains and the next recovery transaction
        // must reacquire source then control before doing work.
        SOURCE_SESSION_LOCKS.with(|locks| locks.borrow_mut().clear());
    }
}

fn ensure_source_session_exit_callback() {
    SOURCE_SESSION_EXIT_CALLBACK_REGISTERED.with(|registered| {
        if registered.replace(true) {
            return;
        }
        unsafe {
            pg_sys::before_shmem_exit(
                Some(source_session_before_shmem_exit),
                pg_sys::Datum::from(0_usize),
            );
            pg_sys::RegisterXactCallback(Some(source_session_top_xact_abort), std::ptr::null_mut());
        };
    });
}

fn release_pending_lock(lock: SourceSessionLock) {
    let removed = SOURCE_SESSION_LOCKS.with(|locks| {
        let mut locks = locks.borrow_mut();
        let before = locks.len();
        locks.retain(|candidate| {
            !same_build_lock(candidate, &lock) || candidate.pending_subid.is_none()
        });
        locks.len() != before
    });
    if removed {
        unlock_build_session_relations(lock);
    }
}

fn release_build_lock(lock: SourceSessionLock) {
    let removed = SOURCE_SESSION_LOCKS.with(|locks| {
        let mut locks = locks.borrow_mut();
        let before = locks.len();
        locks.retain(|candidate| !same_build_lock(candidate, &lock));
        locks.len() != before
    });
    if removed {
        unlock_build_session_relations(lock);
    }
}

fn remove_pending_session_release(id: u64) -> Option<SourceSessionLock> {
    PENDING_SESSION_RELEASES.with(|releases| {
        let mut releases = releases.borrow_mut();
        releases
            .iter()
            .position(|release| release.id == id)
            .map(|position| releases.remove(position).lock)
    })
}

fn schedule_lock_release_after_commit(lock: SourceSessionLock) {
    let id = NEXT_SESSION_RELEASE_ID.with(|next| {
        let id = next.get();
        next.set(id.checked_add(1).expect("session release id exhausted"));
        id
    });
    let pending_subid = unsafe { pg_sys::GetCurrentSubTransactionId() };
    PENDING_SESSION_RELEASES.with(|releases| {
        releases.borrow_mut().push(PendingSessionRelease {
            id,
            lock,
            pending_subid,
        });
    });
    pgrx::register_xact_callback(pgrx::PgXactCallbackEvent::Commit, move || {
        if let Some(lock) = remove_pending_session_release(id) {
            release_build_lock(lock);
        }
    });
    pgrx::register_xact_callback(pgrx::PgXactCallbackEvent::Abort, move || {
        let _ = remove_pending_session_release(id);
    });
    pgrx::register_subxact_callback(
        pgrx::PgSubXactCallbackEvent::CommitSub,
        move |my_subid, parent_subid| {
            PENDING_SESSION_RELEASES.with(|releases| {
                if let Some(release) = releases
                    .borrow_mut()
                    .iter_mut()
                    .find(|release| release.id == id && release.pending_subid == my_subid)
                {
                    release.pending_subid = parent_subid;
                }
            });
        },
    );
    pgrx::register_subxact_callback(
        pgrx::PgSubXactCallbackEvent::AbortSub,
        move |my_subid, _parent_subid| {
            let remove = PENDING_SESSION_RELEASES.with(|releases| {
                releases
                    .borrow()
                    .iter()
                    .any(|release| release.id == id && release.pending_subid == my_subid)
            });
            if remove {
                let _ = remove_pending_session_release(id);
            }
        },
    );
}

pub(crate) fn schedule_session_lock_release_for_control(control_index_oid: pg_sys::Oid) {
    let matching = SOURCE_SESSION_LOCKS.with(|locks| {
        locks
            .borrow()
            .iter()
            .filter(|lock| lock.control_index_oid == control_index_oid)
            .copied()
            .collect::<Vec<_>>()
    });
    for lock in matching {
        schedule_lock_release_after_commit(lock);
    }
}

fn register_source_session_callbacks(lock: SourceSessionLock) {
    pgrx::register_xact_callback(pgrx::PgXactCallbackEvent::Commit, move || {
        SOURCE_SESSION_LOCKS.with(|locks| {
            if let Some(candidate) = locks
                .borrow_mut()
                .iter_mut()
                .find(|candidate| same_build_lock(candidate, &lock))
            {
                candidate.pending_subid = None;
            }
        });
    });
    pgrx::register_xact_callback(pgrx::PgXactCallbackEvent::Abort, move || {
        release_pending_lock(lock);
    });
    pgrx::register_subxact_callback(
        pgrx::PgSubXactCallbackEvent::CommitSub,
        move |my_subid, parent_subid| {
            SOURCE_SESSION_LOCKS.with(|locks| {
                if let Some(candidate) = locks
                    .borrow_mut()
                    .iter_mut()
                    .find(|candidate| same_build_lock(candidate, &lock))
                {
                    if candidate.pending_subid == Some(my_subid) {
                        candidate.pending_subid = Some(parent_subid);
                    }
                }
            });
        },
    );
    pgrx::register_subxact_callback(
        pgrx::PgSubXactCallbackEvent::AbortSub,
        move |my_subid, _parent_subid| {
            let should_release = SOURCE_SESSION_LOCKS.with(|locks| {
                locks.borrow().iter().any(|candidate| {
                    same_build_lock(candidate, &lock) && candidate.pending_subid == Some(my_subid)
                })
            });
            if should_release {
                release_pending_lock(lock)
            }
        },
    );
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn build_session_lock_count_for_test() -> usize {
    SOURCE_SESSION_LOCKS.with(|locks| locks.borrow().len())
}

struct SourceSessionLockGuard {
    lock: SourceSessionLock,
    acquired: bool,
    retained: bool,
}

impl SourceSessionLockGuard {
    fn acquire(
        source_relation_oid: pg_sys::Oid,
        control_index_oid: pg_sys::Oid,
        logical_index_uuid: Uuid,
        build_id: Uuid,
    ) -> Result<Self, String> {
        ensure_source_session_exit_callback();
        let existing = SOURCE_SESSION_LOCKS.with(|locks| {
            locks
                .borrow()
                .iter()
                .find(|lock| {
                    lock.source_relation_oid == source_relation_oid
                        || lock.control_index_oid == control_index_oid
                })
                .copied()
        });
        if let Some(existing) = existing {
            if existing.control_index_oid != control_index_oid
                || existing.logical_index_uuid != logical_index_uuid
                || existing.build_id != build_id
            {
                return Err(
                    "EC_BUILD_STATE: this session already holds the source lock for another build"
                        .to_owned(),
                );
            }
            return Ok(Self {
                lock: existing,
                acquired: false,
                retained: false,
            });
        }
        let pending_subid = unsafe { pg_sys::GetCurrentSubTransactionId() };
        let lock = SourceSessionLock {
            source_relation_oid,
            control_index_oid,
            logical_index_uuid,
            build_id,
            pending_subid: Some(pending_subid),
        };
        // Establish the global source-before-control order with ordinary
        // transaction ownership first. On a busy result PostgreSQL can then
        // release the source lock during ERROR cleanup; manually unlocking a
        // session lock while unwinding a failed SQL call is not safe across
        // backends. Successful acquisition is promoted below to retained
        // session ownership in the same source-before-control order.
        unsafe {
            pg_sys::LockRelationOid(source_relation_oid, pg_sys::ShareLock as pg_sys::LOCKMODE)
        };
        let owns_control_transaction_lock = unsafe {
            pg_sys::ConditionalLockRelationOid(
                control_index_oid,
                pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            )
        };
        if !owns_control_transaction_lock {
            return Err(
                "EC_BUILD_BUSY: another backend owns the coordinator build locks".to_owned(),
            );
        }
        let mut source_lock_id = pg_sys::LockRelId {
            relId: source_relation_oid,
            dbId: unsafe { pg_sys::MyDatabaseId },
        };
        unsafe {
            pg_sys::LockRelationIdForSession(
                &mut source_lock_id,
                pg_sys::ShareLock as pg_sys::LOCKMODE,
            )
        };
        let mut control_lock_id = pg_sys::LockRelId {
            relId: control_index_oid,
            dbId: unsafe { pg_sys::MyDatabaseId },
        };
        unsafe {
            pg_sys::LockRelationIdForSession(
                &mut control_lock_id,
                pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            )
        };
        SOURCE_SESSION_LOCKS.with(|locks| locks.borrow_mut().push(lock));
        register_source_session_callbacks(lock);
        Ok(Self {
            lock,
            acquired: true,
            retained: false,
        })
    }

    fn retain(&mut self) {
        self.retained = true;
    }

    fn release_after_commit(&mut self) {
        let lock = self.lock;
        schedule_lock_release_after_commit(lock);
        // A newly acquired terminal-replay lock must survive until this
        // transaction either commits (the callback releases it) or aborts
        // (the acquisition callback releases it). An already committed lock
        // remains owned on abort because only the commit callback acts.
        self.retained = self.acquired;
    }
}

impl Drop for SourceSessionLockGuard {
    fn drop(&mut self) {
        if !self.acquired || self.retained {
            return;
        }
        unlock_build_session_relations(self.lock);
        SOURCE_SESSION_LOCKS.with(|locks| {
            locks.borrow_mut().retain(|lock| *lock != self.lock);
        });
    }
}

#[derive(Debug, Clone)]
struct DesiredParticipant {
    roster_ordinal: u32,
    node_id: u32,
    endpoint_identity: String,
    conninfo_secret_name: String,
    remote_index_regclass: String,
    participant_logical_index_uuid: Uuid,
    compatibility_digest: [u8; 32],
    is_local: bool,
}

fn fixed_digest(bytes: Vec<u8>, field: &str) -> Result<[u8; 32], String> {
    bytes.try_into().map_err(|bytes: Vec<u8>| {
        format!(
            "EC_BUILD_STATE: {field} is {} bytes, expected 32",
            bytes.len()
        )
    })
}

fn lock_registry_revision(index_oid: pg_sys::Oid, logical_index_uuid: Uuid) -> Result<u64, String> {
    let catalog = extension_relation_name("ec_distann_registry_state")?;
    Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "SELECT revision FROM {catalog}
                      WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                      FOR UPDATE"
                ),
                None,
                &[index_oid.into(), logical_index_uuid.into()],
            )
            .map_err(|_| "EC_BUILD_STATE: registry revision lock failed".to_owned())?
            .map(|row| {
                let revision = row["revision"]
                    .value::<i64>()
                    .map_err(|_| "EC_BUILD_STATE: registry revision decode failed".to_owned())?
                    .ok_or_else(|| "EC_BUILD_STATE: registry revision is NULL".to_owned())?;
                u64::try_from(revision)
                    .map_err(|_| "EC_BUILD_STATE: registry revision is negative".to_owned())
            })
            .next()
            .transpose()?
            .ok_or_else(|| "EC_BUILD_STATE: registry state is absent".to_owned())
    })
}

fn desired_participants(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
) -> Result<Vec<DesiredParticipant>, String> {
    let catalog = extension_relation_name("ec_distann_node_descriptor")?;
    Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT roster_ordinal, node_id, endpoint_identity,
                            conninfo_secret_name, remote_index_regclass,
                            participant_logical_index_uuid, compatibility_digest, is_local
                       FROM {catalog}
                      WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                      ORDER BY roster_ordinal"
                ),
                None,
                &[index_oid.into(), logical_index_uuid.into()],
            )
            .map_err(|_| "EC_BUILD_STATE: desired roster lookup failed".to_owned())?
            .map(|row| {
                let required_i32 = |name: &str| -> Result<i32, String> {
                    row[name]
                        .value::<i32>()
                        .map_err(|_| format!("EC_BUILD_STATE: {name} decode failed"))?
                        .ok_or_else(|| format!("EC_BUILD_STATE: {name} is NULL"))
                };
                let required_string = |name: &str| -> Result<String, String> {
                    row[name]
                        .value::<String>()
                        .map_err(|_| format!("EC_BUILD_STATE: {name} decode failed"))?
                        .ok_or_else(|| format!("EC_BUILD_STATE: {name} is NULL"))
                };
                Ok(DesiredParticipant {
                    roster_ordinal: u32::try_from(required_i32("roster_ordinal")?)
                        .map_err(|_| "EC_BUILD_STATE: roster ordinal is negative".to_owned())?,
                    node_id: u32::try_from(required_i32("node_id")?)
                        .map_err(|_| "EC_BUILD_STATE: node id is negative".to_owned())?,
                    endpoint_identity: required_string("endpoint_identity")?,
                    conninfo_secret_name: required_string("conninfo_secret_name")?,
                    remote_index_regclass: required_string("remote_index_regclass")?,
                    participant_logical_index_uuid: row["participant_logical_index_uuid"]
                        .value::<Uuid>()
                        .map_err(|_| {
                            "EC_BUILD_STATE: participant logical UUID decode failed".to_owned()
                        })?
                        .ok_or_else(|| {
                            "EC_BUILD_STATE: participant logical UUID is NULL".to_owned()
                        })?,
                    compatibility_digest: fixed_digest(
                        row["compatibility_digest"]
                            .value::<Vec<u8>>()
                            .map_err(|_| {
                                "EC_BUILD_STATE: compatibility digest decode failed".to_owned()
                            })?
                            .ok_or_else(|| {
                                "EC_BUILD_STATE: compatibility digest is NULL".to_owned()
                            })?,
                        "compatibility digest",
                    )?,
                    is_local: row["is_local"]
                        .value::<bool>()
                        .map_err(|_| "EC_BUILD_STATE: local flag decode failed".to_owned())?
                        .ok_or_else(|| "EC_BUILD_STATE: local flag is NULL".to_owned())?,
                })
            })
            .collect()
    })
}

fn ensure_build_slot_available(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
) -> Result<(), String> {
    let registration = extension_relation_name("ec_distann_build_registration")?;
    let decision = extension_relation_name("ec_distann_publish_decision")?;
    let occupied = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT
                         EXISTS (
                             SELECT 1 FROM {registration}
                              WHERE index_oid = $1::oid
                                AND logical_index_uuid = $2::uuid
                                AND state IN ('Registered', 'Building', 'Ready', 'Decided')
                         ) OR EXISTS (
                             SELECT 1 FROM {decision}
                              WHERE index_oid = $1::oid
                                AND logical_index_uuid = $2::uuid
                                AND decision_state IN ('Pending', 'Activated')
                         ) AS occupied"
                ),
                None,
                &[index_oid.into(), logical_index_uuid.into()],
            )
            .map_err(|_| "EC_BUILD_STATE: build slot lookup failed".to_owned())?
            .map(|row| {
                row["occupied"]
                    .value::<bool>()
                    .map_err(|_| "EC_BUILD_STATE: build slot decode failed".to_owned())?
                    .ok_or_else(|| "EC_BUILD_STATE: build slot result is NULL".to_owned())
            })
            .next()
            .transpose()?
            .ok_or_else(|| "EC_BUILD_STATE: build slot lookup returned no row".to_owned())
    })?;
    if occupied {
        return Err("EC_BUILD_STATE: another build or publication recovery is active".to_owned());
    }
    Ok(())
}

fn public_roster(participants: &[DesiredParticipant]) -> Result<Vec<DistannRosterEntry>, String> {
    let mut roster = Vec::with_capacity(participants.len());
    for (ordinal, participant) in participants.iter().enumerate() {
        if participant.roster_ordinal as usize != ordinal {
            return Err("EC_NODE_DESCRIPTOR: roster ordinals must be dense from zero".to_owned());
        }
        roster.push(DistannRosterEntry {
            node_id: participant.node_id,
            logical_index_uuid: *participant.participant_logical_index_uuid.as_bytes(),
            endpoint_identity: participant.endpoint_identity.clone(),
        });
    }
    validate_roster(&roster)?;
    Ok(roster)
}

fn encode_roster_snapshot(roster: &[DistannRosterEntry]) -> Result<Vec<u8>, String> {
    let mut encoder = CanonicalEncoder::with_capacity(64 * roster.len());
    encoder.put_u16(BUILD_ROSTER_SNAPSHOT_VERSION);
    encoder.put_u32(
        u32::try_from(roster.len())
            .map_err(|_| "EC_NODE_DESCRIPTOR: roster count exceeds u32".to_owned())?,
    );
    for entry in roster {
        encoder.put_u32(entry.node_id);
        encoder.put_fixed(&entry.logical_index_uuid);
        encoder.put_string(&entry.endpoint_identity)?;
    }
    encoder.finish()
}

#[allow(clippy::too_many_arguments)]
fn encode_registration(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    source_relation_oid: pg_sys::Oid,
    epoch: u64,
    build_id: Uuid,
    registry_revision: u64,
    roster_snapshot: &[u8],
    roster_digest: [u8; 32],
    row_schema_fingerprint: [u8; 32],
    compatibility_digest: [u8; 32],
    participants: &[DesiredParticipant],
) -> Result<Vec<u8>, String> {
    let mut encoder = CanonicalEncoder::with_capacity(
        192 + roster_snapshot.len() + participants.len().saturating_mul(160),
    );
    encoder.put_u16(BUILD_REGISTRATION_VERSION);
    encoder.put_u32(u32::from(index_oid));
    encoder.put_fixed(logical_index_uuid.as_bytes());
    encoder.put_u32(u32::from(source_relation_oid));
    encoder.put_u64(epoch);
    encoder.put_fixed(build_id.as_bytes());
    encoder.put_u64(registry_revision);
    encoder.put_len_prefixed(roster_snapshot)?;
    encoder.put_fixed(&roster_digest);
    encoder.put_fixed(&row_schema_fingerprint);
    encoder.put_fixed(&compatibility_digest);
    encoder.put_u32(
        u32::try_from(participants.len())
            .map_err(|_| "EC_BUILD_STATE: participant count exceeds u32".to_owned())?,
    );
    for participant in participants {
        encoder.put_u32(participant.roster_ordinal);
        encoder.put_u32(participant.node_id);
        encoder.put_string(&participant.endpoint_identity)?;
        encoder.put_string(&participant.conninfo_secret_name)?;
        encoder.put_string(&participant.remote_index_regclass)?;
        encoder.put_fixed(participant.participant_logical_index_uuid.as_bytes());
        encoder.put_fixed(&participant.compatibility_digest);
        encoder.put_u8(u8::from(participant.is_local));
    }
    encoder.finish()
}

#[allow(clippy::too_many_arguments)]
fn registration_digest(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    source_relation_oid: pg_sys::Oid,
    epoch: u64,
    build_id: Uuid,
    registry_revision: u64,
    roster_snapshot: &[u8],
    roster_digest: [u8; 32],
    row_schema_fingerprint: [u8; 32],
    compatibility_digest: [u8; 32],
    participants: &[DesiredParticipant],
) -> Result<[u8; 32], String> {
    Ok(domain_digest(
        BUILD_REGISTRATION_DOMAIN,
        &encode_registration(
            index_oid,
            logical_index_uuid,
            source_relation_oid,
            epoch,
            build_id,
            registry_revision,
            roster_snapshot,
            roster_digest,
            row_schema_fingerprint,
            compatibility_digest,
            participants,
        )?,
    ))
}

#[derive(Debug)]
struct StoredRegistration {
    source_relation_oid: pg_sys::Oid,
    epoch: u64,
    state: String,
    registry_revision: u64,
    roster_snapshot: Vec<u8>,
    roster_digest: [u8; 32],
    row_schema_fingerprint: [u8; 32],
    registration_digest: [u8; 32],
}

fn replay_registration(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
    epoch: u64,
    current_row_schema_fingerprint: [u8; 32],
    current_compatibility_digest: [u8; 32],
    source_relation_oid: pg_sys::Oid,
) -> Result<Option<([u8; 32], bool)>, String> {
    let registration = extension_relation_name("ec_distann_build_registration")?;
    let binding = extension_relation_name("ec_distann_build_participant_binding")?;
    let stored = Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "SELECT source_relid, epoch, state, registry_revision, roster_snapshot,
                            roster_digest, row_schema_fingerprint,
                            registration_digest
                       FROM {registration}
                      WHERE index_oid = $1::oid
                        AND logical_index_uuid = $2::uuid
                        AND build_id = $3::uuid
                      FOR UPDATE"
                ),
                None,
                &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
            )
            .map_err(|_| "EC_BUILD_STATE: registration replay lookup failed".to_owned())?
            .map(|row| {
                let stored_epoch = row["epoch"]
                    .value::<i64>()
                    .map_err(|_| "EC_BUILD_STATE: registration epoch decode failed".to_owned())?
                    .ok_or_else(|| "EC_BUILD_STATE: registration epoch is NULL".to_owned())?;
                let registry_revision = row["registry_revision"]
                    .value::<i64>()
                    .map_err(|_| "EC_BUILD_STATE: registry revision decode failed".to_owned())?
                    .ok_or_else(|| "EC_BUILD_STATE: registry revision is NULL".to_owned())?;
                Ok::<StoredRegistration, String>(StoredRegistration {
                    source_relation_oid: row["source_relid"]
                        .value::<pg_sys::Oid>()
                        .map_err(|_| "EC_BUILD_STATE: source relation decode failed".to_owned())?
                        .ok_or_else(|| "EC_BUILD_STATE: source relation is NULL".to_owned())?,
                    epoch: u64::try_from(stored_epoch).map_err(|_| {
                        "EC_BUILD_STATE: registration epoch is not positive".to_owned()
                    })?,
                    state: row["state"]
                        .value::<String>()
                        .map_err(|_| "EC_BUILD_STATE: registration state decode failed".to_owned())?
                        .ok_or_else(|| "EC_BUILD_STATE: registration state is NULL".to_owned())?,
                    registry_revision: u64::try_from(registry_revision)
                        .map_err(|_| "EC_BUILD_STATE: registry revision is negative".to_owned())?,
                    roster_snapshot: row["roster_snapshot"]
                        .value::<Vec<u8>>()
                        .map_err(|_| "EC_BUILD_STATE: roster snapshot decode failed".to_owned())?
                        .ok_or_else(|| "EC_BUILD_STATE: roster snapshot is NULL".to_owned())?,
                    roster_digest: fixed_digest(
                        row["roster_digest"]
                            .value::<Vec<u8>>()
                            .map_err(|_| "EC_BUILD_STATE: roster digest decode failed".to_owned())?
                            .ok_or_else(|| "EC_BUILD_STATE: roster digest is NULL".to_owned())?,
                        "roster digest",
                    )?,
                    row_schema_fingerprint: fixed_digest(
                        row["row_schema_fingerprint"]
                            .value::<Vec<u8>>()
                            .map_err(|_| {
                                "EC_BUILD_STATE: row schema fingerprint decode failed".to_owned()
                            })?
                            .ok_or_else(|| {
                                "EC_BUILD_STATE: row schema fingerprint is NULL".to_owned()
                            })?,
                        "row schema fingerprint",
                    )?,
                    registration_digest: fixed_digest(
                        row["registration_digest"]
                            .value::<Vec<u8>>()
                            .map_err(|_| {
                                "EC_BUILD_STATE: registration digest decode failed".to_owned()
                            })?
                            .ok_or_else(|| {
                                "EC_BUILD_STATE: registration digest is NULL".to_owned()
                            })?,
                        "registration digest",
                    )?,
                })
            })
            .next()
            .transpose()
    })?;
    let Some(stored) = stored else {
        return Ok(None);
    };
    if stored.epoch != epoch {
        return Err("EC_BUILD_ID_CONFLICT: build id has a different epoch".to_owned());
    }
    if stored.source_relation_oid != source_relation_oid {
        return Err("EC_BUILD_ID_CONFLICT: build id has a different source relation".to_owned());
    }
    if stored.state == "Aborted" {
        return Err("EC_BUILD_ID_CONFLICT: build id is already Aborted".to_owned());
    }
    if stored.row_schema_fingerprint != current_row_schema_fingerprint {
        return Err(
            "EC_BUILD_ID_CONFLICT: source row schema changed after registration".to_owned(),
        );
    }

    let participants = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT roster_ordinal, node_id, endpoint_identity,
                            conninfo_secret_name, remote_index_regclass,
                            participant_logical_index_uuid, compatibility_digest, is_local
                       FROM {binding}
                      WHERE index_oid = $1::oid
                        AND logical_index_uuid = $2::uuid
                        AND build_id = $3::uuid
                      ORDER BY roster_ordinal"
                ),
                None,
                &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
            )
            .map_err(|_| "EC_BUILD_STATE: participant binding replay lookup failed".to_owned())?
            .map(|row| {
                let required_i32 = |name: &str| -> Result<i32, String> {
                    row[name]
                        .value::<i32>()
                        .map_err(|_| format!("EC_BUILD_STATE: {name} decode failed"))?
                        .ok_or_else(|| format!("EC_BUILD_STATE: {name} is NULL"))
                };
                let required_string = |name: &str| -> Result<String, String> {
                    row[name]
                        .value::<String>()
                        .map_err(|_| format!("EC_BUILD_STATE: {name} decode failed"))?
                        .ok_or_else(|| format!("EC_BUILD_STATE: {name} is NULL"))
                };
                Ok(DesiredParticipant {
                    roster_ordinal: u32::try_from(required_i32("roster_ordinal")?)
                        .map_err(|_| "EC_BUILD_STATE: roster ordinal is negative".to_owned())?,
                    node_id: u32::try_from(required_i32("node_id")?)
                        .map_err(|_| "EC_BUILD_STATE: node id is negative".to_owned())?,
                    endpoint_identity: required_string("endpoint_identity")?,
                    conninfo_secret_name: required_string("conninfo_secret_name")?,
                    remote_index_regclass: required_string("remote_index_regclass")?,
                    participant_logical_index_uuid: row["participant_logical_index_uuid"]
                        .value::<Uuid>()
                        .map_err(|_| {
                            "EC_BUILD_STATE: participant logical UUID decode failed".to_owned()
                        })?
                        .ok_or_else(|| {
                            "EC_BUILD_STATE: participant logical UUID is NULL".to_owned()
                        })?,
                    compatibility_digest: fixed_digest(
                        row["compatibility_digest"]
                            .value::<Vec<u8>>()
                            .map_err(|_| {
                                "EC_BUILD_STATE: compatibility digest decode failed".to_owned()
                            })?
                            .ok_or_else(|| {
                                "EC_BUILD_STATE: compatibility digest is NULL".to_owned()
                            })?,
                        "compatibility digest",
                    )?,
                    is_local: row["is_local"]
                        .value::<bool>()
                        .map_err(|_| "EC_BUILD_STATE: local flag decode failed".to_owned())?
                        .ok_or_else(|| "EC_BUILD_STATE: local flag is NULL".to_owned())?,
                })
            })
            .collect::<Result<Vec<_>, String>>()
    })?;
    if participants.is_empty() {
        return Err("EC_BUILD_STATE: durable registration has no participant bindings".to_owned());
    }
    if participants
        .iter()
        .any(|participant| participant.compatibility_digest != current_compatibility_digest)
    {
        return Err(
            "EC_BUILD_ID_CONFLICT: participant compatibility changed after registration".to_owned(),
        );
    }
    let roster = public_roster(&participants)?;
    let roster_snapshot = encode_roster_snapshot(&roster)?;
    let frozen_roster_digest = roster_digest(&roster)?;
    if roster_snapshot != stored.roster_snapshot || frozen_roster_digest != stored.roster_digest {
        return Err(
            "EC_BUILD_STATE: durable registration roster is internally inconsistent".to_owned(),
        );
    }
    let expected_digest = registration_digest(
        index_oid,
        logical_index_uuid,
        stored.source_relation_oid,
        stored.epoch,
        build_id,
        stored.registry_revision,
        &stored.roster_snapshot,
        stored.roster_digest,
        stored.row_schema_fingerprint,
        current_compatibility_digest,
        &participants,
    )?;
    if expected_digest != stored.registration_digest {
        return Err("EC_BUILD_STATE: durable registration digest is inconsistent".to_owned());
    }
    Ok(Some((
        stored.registration_digest,
        stored.state != "Published",
    )))
}

#[pg_extern(volatile, strict)]
fn ec_distann_begin_epoch_build(index_regclass: PgRelation, epoch: i64, build_id: Uuid) -> Vec<u8> {
    (|| -> Result<Vec<u8>, String> {
        super::build_gate::require_shared_preload()?;
        super::build_gate::lock_global_gate_serialization(false)?;
        let epoch = u64::try_from(epoch)
            .ok()
            .filter(|epoch| *epoch > 0)
            .ok_or_else(|| "EC_BUILD_STATE: epoch must be positive".to_owned())?;
        if !is_rfc4122_v4_uuid(build_id.as_bytes()) {
            return Err("EC_BUILD_ID_CONFLICT: build id must be RFC 4122 v4".to_owned());
        }
        let index_oid = index_regclass.oid();
        // PgRelation conversion holds AccessShareLock. Use it only for the
        // short preflight, then release every control-index lock before taking
        // the source session lock. Source DDL otherwise has the inverse
        // source→index order and can deadlock begin-build.
        let (preflight_control, preflight_handle, _metadata, preflight_uuid) =
            open_control_index(
                index_oid,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
                "ec_distann_begin_epoch_build preflight",
            )?;
        let source_relation_oid = index_heap_relation_oid_handle(preflight_handle);
        drop(preflight_control);
        drop(index_regclass);

        let mut source_lock = SourceSessionLockGuard::acquire(
            source_relation_oid,
            index_oid,
            preflight_uuid,
            build_id,
        )?;
        let has_inheritance_edge = Spi::get_one_with_args::<bool>(
            "SELECT EXISTS (
                 SELECT 1 FROM pg_catalog.pg_inherits
                  WHERE inhrelid = $1::oid OR inhparent = $1::oid
             )",
            &[source_relation_oid.into()],
        )
        .map_err(|_| "EC_BUILD_STATE: source inheritance topology lookup failed".to_owned())?
        .ok_or_else(|| "EC_BUILD_STATE: source inheritance topology lookup returned NULL".to_owned())?;
        if has_inheritance_edge {
            return Err(
                "EC_BUILD_STATE: distributed build sources may not be partitioned or participate in table inheritance in format v1"
                    .to_owned(),
            );
        }
        let (mut control, handle, metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_begin_epoch_build",
        )?;
        if logical_index_uuid != preflight_uuid
            || index_heap_relation_oid_handle(handle) != source_relation_oid
        {
            return Err(
                "EC_BUILD_ID_CONFLICT: control identity changed while acquiring source lock"
                    .to_owned(),
            );
        }
        control.retain_lock_until_transaction_end();
        // Registry operations and begin-build share this row lock after the
        // retained coordinator-control relation lock. This includes replay.
        let registry_revision = lock_registry_revision(index_oid, logical_index_uuid)?;
        let row_schema_fingerprint = resolve_relation_schema(source_relation_oid)?
            .descriptor
            .fingerprint()?;
        let compatibility_digest = control_compatibility_digest(handle, &metadata)?;

        if let Some((existing, requires_source_lock)) = replay_registration(
            index_oid,
            logical_index_uuid,
            build_id,
            epoch,
            row_schema_fingerprint,
            compatibility_digest,
            source_relation_oid,
        )? {
            if requires_source_lock {
                source_lock.retain();
            } else {
                source_lock.release_after_commit();
            }
            return Ok(existing.to_vec());
        }

        ensure_build_slot_available(index_oid, logical_index_uuid)?;

        let participants = desired_participants(index_oid, logical_index_uuid)?;
        if participants.is_empty() {
            return Err("EC_NODE_DESCRIPTOR: cannot register a build with no owners".to_owned());
        }
        let roster = public_roster(&participants)?;
        let roster_snapshot = encode_roster_snapshot(&roster)?;
        let frozen_roster_digest = roster_digest(&roster)?;
        if participants
            .iter()
            .any(|participant| participant.compatibility_digest != compatibility_digest)
        {
            return Err(
                "EC_NODE_DESCRIPTOR: desired participant compatibility changed before build registration"
                    .to_owned(),
            );
        }
        let digest = registration_digest(
            index_oid,
            logical_index_uuid,
            source_relation_oid,
            epoch,
            build_id,
            registry_revision,
            &roster_snapshot,
            frozen_roster_digest,
            row_schema_fingerprint,
            compatibility_digest,
            &participants,
        )?;
        let registration = extension_relation_name("ec_distann_build_registration")?;
        let binding = extension_relation_name("ec_distann_build_participant_binding")?;
        Spi::connect_mut(|client| -> Result<(), String> {
            client
                .update(
                    &format!(
                        "INSERT INTO {registration} (
                             index_oid, logical_index_uuid, source_relid, build_id, epoch, state,
                             registry_revision, roster_snapshot, roster_digest,
                             row_schema_fingerprint, registration_digest
                         ) VALUES (
                             $1::oid, $2::uuid, $3::oid, $4::uuid, $5::bigint, 'Registered',
                             $6::bigint, $7::bytea, $8::bytea, $9::bytea, $10::bytea
                         )"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        source_relation_oid.into(),
                        build_id.into(),
                        i64::try_from(epoch)
                            .map_err(|_| "EC_BUILD_STATE: epoch exceeds bigint".to_owned())?
                            .into(),
                        i64::try_from(registry_revision)
                            .map_err(|_| {
                                "EC_BUILD_STATE: registry revision exceeds bigint".to_owned()
                            })?
                            .into(),
                        roster_snapshot.clone().into(),
                        frozen_roster_digest.to_vec().into(),
                        row_schema_fingerprint.to_vec().into(),
                        digest.to_vec().into(),
                    ],
                )
                .map_err(|_| "EC_BUILD_STATE: build registration insert failed".to_owned())?;
            for participant in &participants {
                client
                    .update(
                        &format!(
                            "INSERT INTO {binding} (
                                 index_oid, logical_index_uuid, build_id, roster_ordinal,
                                 node_id, endpoint_identity, conninfo_secret_name,
                                 remote_index_regclass, participant_logical_index_uuid,
                                 compatibility_digest, is_local
                             ) VALUES (
                                 $1::oid, $2::uuid, $3::uuid, $4::integer,
                                 $5::integer, $6::text, $7::text, $8::text,
                                 $9::uuid, $10::bytea, $11::boolean
                             )"
                        ),
                        None,
                        &[
                            index_oid.into(),
                            logical_index_uuid.into(),
                            build_id.into(),
                            i32::try_from(participant.roster_ordinal)
                                .expect("validated roster ordinal fits i32")
                                .into(),
                            i32::try_from(participant.node_id)
                                .expect("validated node id fits i32")
                                .into(),
                            participant.endpoint_identity.clone().into(),
                            participant.conninfo_secret_name.clone().into(),
                            participant.remote_index_regclass.clone().into(),
                            participant.participant_logical_index_uuid.into(),
                            participant.compatibility_digest.to_vec().into(),
                            participant.is_local.into(),
                        ],
                    )
                    .map_err(|_| {
                        "EC_BUILD_STATE: private participant binding insert failed".to_owned()
                    })?;
            }
            Ok(())
        })?;
        source_lock.retain();
        Ok(digest.to_vec())
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"))
}

/// Capture the coordinator's frozen source MVCC snapshot as a canonical
/// `DistannSourceSnapshot` (FR-082). The active snapshot's 32-bit transaction
/// ids are widened to full wrap-safe 64-bit ids against `ReadNextFullTransactionId`,
/// matching PostgreSQL's `FullTransactionIdFromAllowableAt`; the in-progress
/// arrays are sorted into the canonical strictly-ascending order the encoding
/// requires. build_epoch takes this once, before the counting/digest pass.
fn capture_source_snapshot() -> Result<DistannSourceSnapshot, String> {
    let snapshot_ptr = unsafe { pg_sys::GetActiveSnapshot() };
    if snapshot_ptr.is_null() {
        return Err("EC_SOURCE_SNAPSHOT: build_epoch has no active PostgreSQL snapshot".to_owned());
    }
    let snapshot = unsafe { &*snapshot_ptr };
    let next_full = unsafe { pg_sys::ReadNextFullTransactionId() }.value;
    let next_lo = next_full as u32;
    let next_epoch = (next_full >> 32) as u32;
    // A read snapshot's ids all precede nextXid, so place each in the epoch that
    // keeps its full id at or below nextFullXid.
    let to_full = |xid: pg_sys::TransactionId| -> u64 {
        let xid = xid.into_inner();
        let epoch = if xid > next_lo {
            next_epoch.wrapping_sub(1)
        } else {
            next_epoch
        };
        (u64::from(epoch) << 32) | u64::from(xid)
    };
    let read_full_array = |base: *mut pg_sys::TransactionId, count: usize| -> Vec<u64> {
        let mut ids = Vec::with_capacity(count);
        if !base.is_null() {
            for offset in 0..count {
                ids.push(to_full(unsafe { *base.add(offset) }));
            }
        }
        // The snapshot arrays are unordered sets; canonical encoding requires a
        // strictly-ascending order and the members are distinct in-progress ids.
        ids.sort_unstable();
        ids
    };

    let xcnt = usize::try_from(snapshot.xcnt)
        .map_err(|_| "EC_SOURCE_SNAPSHOT: in-progress xid count is invalid".to_owned())?;
    let xip = read_full_array(snapshot.xip, xcnt);
    let subxip = if snapshot.suboverflowed {
        Vec::new()
    } else {
        let subxcnt = usize::try_from(snapshot.subxcnt)
            .map_err(|_| "EC_SOURCE_SNAPSHOT: subtransaction xid count is invalid".to_owned())?;
        read_full_array(snapshot.subxip, subxcnt)
    };

    let database_name = {
        let name_ptr = unsafe { pg_sys::get_database_name(pg_sys::MyDatabaseId) };
        if name_ptr.is_null() {
            return Err("EC_SOURCE_SNAPSHOT: current database name is unavailable".to_owned());
        }
        let name = unsafe { std::ffi::CStr::from_ptr(name_ptr) }
            .to_str()
            .map_err(|_| "EC_SOURCE_SNAPSHOT: database name is not UTF-8".to_owned())?
            .to_owned();
        unsafe { pg_sys::pfree(name_ptr.cast()) };
        name
    };

    let captured = DistannSourceSnapshot {
        system_identifier: unsafe { pg_sys::GetSystemIdentifier() },
        database_name,
        xmin_full: to_full(snapshot.xmin),
        xmax_full: to_full(snapshot.xmax),
        curcid: snapshot.curcid,
        xip,
        subxip,
        suboverflowed: snapshot.suboverflowed,
        taken_during_recovery: snapshot.takenDuringRecovery,
    };
    captured.validate()?;
    Ok(captured)
}

/// Build a distributed epoch to Ready in one coordinator transaction and return
/// the 32-byte candidate manifest digest (FR-078:275-323). Requires the durable
/// registration from `ec_distann_begin_epoch_build`. Captures one frozen source
/// snapshot, builds the physical graph workspace, makes the counting/digest pass
/// binding per-owner and global identities, drives the single local participant
/// begin/stage/seal, then atomically persists the immutable build candidate and
/// transitions the registration to Ready. This slice supports a single local
/// participant; multi-owner remote transport is a later slice.
#[pg_extern(volatile, strict)]
fn ec_distann_build_epoch(index_regclass: PgRelation, epoch: i64, build_id: Uuid) -> Vec<u8> {
    (|| -> Result<Vec<u8>, String> {
        super::build_gate::require_shared_preload()?;
        let epoch_u64 = u64::try_from(epoch)
            .ok()
            .filter(|value| *value > 0)
            .ok_or_else(|| "EC_BUILD_STATE: epoch must be positive".to_owned())?;
        if !is_rfc4122_v4_uuid(build_id.as_bytes()) {
            return Err("EC_BUILD_ID_CONFLICT: build id must be an RFC 4122 v4 UUID".to_owned());
        }
        let index_oid = index_regclass.oid();

        // One frozen source MVCC snapshot for the whole build.
        let source_snapshot = capture_source_snapshot()?;
        let source_snapshot_digest = source_snapshot.digest()?;

        let (mut control, handle, metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_build_epoch",
        )?;
        control.retain_lock_until_transaction_end();
        let source_relation_oid = index_heap_relation_oid_handle(handle);
        let _registry_revision = lock_registry_revision(index_oid, logical_index_uuid)?;
        let row_schema = resolve_relation_schema(source_relation_oid)?.descriptor;
        let row_schema_fingerprint = row_schema.fingerprint()?;
        let compatibility_digest = control_compatibility_digest(handle, &metadata)?;

        // build_epoch consumes the durable registration written by begin.
        let (registration_digest, _requires_source_lock) = replay_registration(
            index_oid,
            logical_index_uuid,
            build_id,
            epoch_u64,
            row_schema_fingerprint,
            compatibility_digest,
            source_relation_oid,
        )?
        .ok_or_else(|| {
            "EC_BUILD_STATE: build_epoch requires a durable registration from begin_epoch_build"
                .to_owned()
        })?;

        // Reacquire the frozen roster (revision-locked so it matches registration).
        let participants = desired_participants(index_oid, logical_index_uuid)?;
        let roster = public_roster(&participants)?;
        let roster_digest_bytes = roster_digest(&roster)?;
        if participants.len() != 1 || !participants[0].is_local {
            return Err(
                "EC_BUILD_STATE: build_epoch currently supports only a single local participant"
                    .to_owned(),
            );
        }

        // Capture source rows and build the physical graph workspace.
        let index_relation = crate::storage::relation_guard::IndexRelationGuard::open(
            index_oid,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            "ec_distann_build_epoch",
        );
        let heap = crate::storage::relation_guard::HeapRelationGuard::try_access_share(
            index_relation.heap_relation_oid(),
        )
        .ok_or_else(|| "EC_BUILD_INCOMPLETE: source heap could not open".to_owned())?;
        let index_info = crate::am::common::index_info::IndexInfoGuard::build(
            index_relation.as_ptr(),
            "ec_distann_build_epoch",
        );
        let capture = unsafe {
            super::capture_physical_source_rows(
                heap.as_ptr(),
                index_relation.as_ptr(),
                index_info.as_ptr(),
            )
        }?;
        let mut workspace =
            super::build_physical_graph_workspace(index_relation.as_ptr(), capture)?;

        // Counting/digest pass before the first participant begin.
        let expectations =
            workspace.owner_expectations(&roster, super::DISTANN_PLACEMENT_HASH_VERSION)?;
        let (global_count, global_graph_digest, global_row_tier_digest) =
            workspace.global_digests()?;
        // The coordinator head sample is not derived in this slice; both the
        // build spec and manifest carry the same value so the candidate chain
        // round-trips through the publish-decision re-verification.
        let head_sample_digest = [0u8; 32];

        let options = super::options::relation_options(index_relation.as_ptr());
        let to_u16 = |value: i32, field: &str| -> Result<u16, String> {
            u16::try_from(value).map_err(|_| format!("EC_BUILD_STATE: {field} out of range"))
        };
        let to_u32 = |value: i32, field: &str| -> Result<u32, String> {
            u32::try_from(value).map_err(|_| format!("EC_BUILD_STATE: {field} out of range"))
        };
        let build_options = DistannBuildOptions {
            build_list_size: to_u16(options.build_list_size, "build_list_size")?,
            alpha: options.alpha,
            seed: metadata.seed,
            closure_epsilon: options.closure_epsilon,
            head_index_cap: to_u32(options.head_index_cap, "head_index_cap")?,
            build_shards: to_u32(options.build_shards, "build_shards")?,
        };

        let codec_artifact = workspace.codec_artifact().clone();
        let dimensions = to_u16(i32::from(codec_artifact.dimensions()), "dimensions")?;
        let shape = workspace.shape();
        let code_stride = to_u32(
            i32::try_from(shape.code_stride)
                .map_err(|_| "EC_BUILD_STATE: code stride out of range".to_owned())?,
            "code_stride",
        )?;

        let descriptor = DistannGenerationDescriptor {
            coordinator_logical_index_uuid: metadata.logical_index_uuid,
            index_format_version: super::DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
            graph_record_version: super::DISTANN_GRAPH_RECORD_VERSION,
            handoff_wire_version: super::DISTANN_HANDOFF_WIRE_VERSION,
            dimensions,
            graph_degree: metadata.graph_degree_r,
            placement_hash_version: super::DISTANN_PLACEMENT_HASH_VERSION,
            roster: roster.clone(),
            neighbor_codec_kind: metadata.neighbor_codec_kind,
            codec_artifact,
            row_schema,
        };
        let descriptor_bytes = descriptor.encode()?;
        let descriptor_digest = descriptor.digest()?;

        let build_spec = DistannBuildSpec {
            epoch: epoch_u64,
            build_id: *build_id.as_bytes(),
            parent_fingerprint: Vec::new(),
            source_snapshot_digest,
            generation_descriptor_digest: descriptor_digest,
            build_options: build_options.clone(),
            expected_global_count: global_count,
            expected_global_graph_digest: global_graph_digest,
            expected_global_row_tier_digest: global_row_tier_digest,
            head_sample_digest,
            owner_expectations: expectations.clone(),
        };
        let build_spec_digest = build_spec.digest()?;

        // Drive the single local participant: begin -> stage -> seal.
        let local = expectations[0].clone();
        let local_owner_count = i64::try_from(local.expected_count)
            .map_err(|_| "EC_BUILD_STATE: owner count exceeds bigint".to_owned())?;
        let begin_fn = extension_relation_name("ec_distann_begin_epoch_handoff")?;
        Spi::connect_mut(|client| -> Result<(), String> {
            client
                .update(
                    &format!(
                        "SELECT state FROM {begin_fn}(
                             $1::oid::regclass, $2::bigint, $3::uuid, $4::bytea, $5::bytea,
                             $6::bytea, $7::bytea, $8::bigint, $9::bytea
                         )"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        epoch.into(),
                        build_id.into(),
                        build_spec_digest.to_vec().into(),
                        roster_digest_bytes.to_vec().into(),
                        descriptor_bytes.clone().into(),
                        descriptor_digest.to_vec().into(),
                        local_owner_count.into(),
                        local.expected_owner_digest.to_vec().into(),
                    ],
                )
                .map_err(|_| "EC_BUILD_INCOMPLETE: begin generation dispatch failed".to_owned())?;
            Ok(())
        })?;

        let route_identity = DistannHandoffRouteIdentity {
            epoch: epoch_u64,
            build_id: *build_id.as_bytes(),
            build_spec_digest,
            row_schema_fingerprint,
            index_format_version: super::DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
            neighbor_codec_kind: metadata.neighbor_codec_kind,
            placement_hash_version: super::DISTANN_PLACEMENT_HASH_VERSION,
        };
        let stage_fn = extension_relation_name("ec_distann_stage_epoch_batch")?;
        workspace.route(route_identity, 1, &mut |_owner, sequence, digest, encoded| {
            let sequence = i64::try_from(sequence)
                .map_err(|_| "EC_BUILD_STATE: batch sequence exceeds bigint".to_owned())?;
            Spi::connect_mut(|client| -> Result<DistannStageAck, String> {
                let row = client
                    .update(
                        &format!(
                            "SELECT accepted_record_count, cumulative_record_count,
                                    cumulative_owner_digest
                               FROM {stage_fn}(
                                   $1::oid::regclass, $2::uuid, $3::bigint, $4::bytea, $5::bytea
                               )"
                        ),
                        None,
                        &[
                            index_oid.into(),
                            build_id.into(),
                            sequence.into(),
                            digest.to_vec().into(),
                            encoded.to_vec().into(),
                        ],
                    )
                    .map_err(|_| "EC_BUILD_INCOMPLETE: stage batch dispatch failed".to_owned())?
                    .next()
                    .ok_or_else(|| "EC_BUILD_INCOMPLETE: stage batch returned no row".to_owned())?;
                let accepted = row["accepted_record_count"]
                    .value::<i64>()
                    .map_err(|_| "EC_BUILD_INCOMPLETE: accepted count decode failed".to_owned())?
                    .ok_or_else(|| "EC_BUILD_INCOMPLETE: accepted count is NULL".to_owned())?;
                let cumulative = row["cumulative_record_count"]
                    .value::<i64>()
                    .map_err(|_| "EC_BUILD_INCOMPLETE: cumulative count decode failed".to_owned())?
                    .ok_or_else(|| "EC_BUILD_INCOMPLETE: cumulative count is NULL".to_owned())?;
                let cumulative_owner_digest = row["cumulative_owner_digest"]
                    .value::<Vec<u8>>()
                    .map_err(|_| "EC_BUILD_INCOMPLETE: cumulative digest decode failed".to_owned())?
                    .ok_or_else(|| "EC_BUILD_INCOMPLETE: cumulative digest is NULL".to_owned())?;
                Ok(DistannStageAck {
                    accepted_record_count: u64::try_from(accepted).map_err(|_| {
                        "EC_BUILD_INCOMPLETE: accepted count is negative".to_owned()
                    })?,
                    cumulative_record_count: u64::try_from(cumulative).map_err(|_| {
                        "EC_BUILD_INCOMPLETE: cumulative count is negative".to_owned()
                    })?,
                    cumulative_owner_digest: cumulative_owner_digest.try_into().map_err(|_| {
                        "EC_BUILD_INCOMPLETE: cumulative digest is not 32 bytes".to_owned()
                    })?,
                })
            })
        })?;

        let seal_fn = extension_relation_name("ec_distann_seal_epoch_handoff")?;
        let receipt_bytes = Spi::connect_mut(|client| -> Result<Vec<u8>, String> {
            client
                .update(
                    &format!(
                        "SELECT {seal_fn}(
                             $1::oid::regclass, $2::uuid, $3::bigint, $4::bytea
                         ) AS ready_receipt"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        build_id.into(),
                        local_owner_count.into(),
                        local.expected_owner_digest.to_vec().into(),
                    ],
                )
                .map_err(|_| "EC_BUILD_INCOMPLETE: seal dispatch failed".to_owned())?
                .map(|row| {
                    row["ready_receipt"]
                        .value::<Vec<u8>>()
                        .map_err(|_| "EC_BUILD_INCOMPLETE: ready receipt decode failed".to_owned())?
                        .ok_or_else(|| "EC_BUILD_INCOMPLETE: ready receipt is NULL".to_owned())
                })
                .next()
                .transpose()?
                .ok_or_else(|| "EC_BUILD_INCOMPLETE: seal returned no receipt".to_owned())
        })?;
        let receipt = super::manifest_v2::DistannReadyReceipt::decode(&receipt_bytes)?;

        let codec_parameters = DistannManifestCodecParameters {
            codec_kind: metadata.neighbor_codec_kind,
            dimensions,
            code_stride,
            seed: metadata.seed,
            transform_dim: 0,
            group_count: 0,
            group_size: 0,
            centroids_per_group: 0,
        };
        let manifest = DistannEpochManifestV2 {
            epoch: epoch_u64,
            build_id: *build_id.as_bytes(),
            parent_fingerprint: Vec::new(),
            source_snapshot_digest,
            build_spec_digest,
            generation_descriptor_digest: descriptor_digest,
            placement_hash_version: super::DISTANN_PLACEMENT_HASH_VERSION,
            roster: roster.clone(),
            index_format_version: super::DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
            graph_record_version: super::DISTANN_GRAPH_RECORD_VERSION,
            handoff_wire_version: super::DISTANN_HANDOFF_WIRE_VERSION,
            codec_parameters,
            build_options: DistannManifestBuildOptions {
                graph_degree: metadata.graph_degree_r,
                options: build_options,
            },
            row_schema_fingerprint,
            head_sample_digest,
            global_record_count: global_count,
            global_graph_digest,
            global_row_tier_digest,
            participant_receipts: vec![receipt],
        };

        let candidate = DistannBuildCandidateV1::from_components(
            registration_digest,
            &build_spec,
            &descriptor,
            &source_snapshot,
            &manifest,
        )?;
        let candidate_digest = candidate.digest()?;

        // Atomically persist the immutable candidate and mark the registration
        // Ready in this coordinator transaction.
        let candidate_table = extension_relation_name("ec_distann_build_candidate")?;
        let registration = extension_relation_name("ec_distann_build_registration")?;
        Spi::connect_mut(|client| -> Result<(), String> {
            client
                .update(
                    &format!(
                        "INSERT INTO {candidate_table} (
                             index_oid, logical_index_uuid, build_id, epoch,
                             registration_digest, build_spec, build_spec_digest,
                             generation_descriptor, generation_descriptor_digest,
                             source_snapshot, source_snapshot_digest,
                             ready_receipt_set, ready_receipt_set_digest,
                             epoch_manifest, manifest_digest, epoch_fingerprint,
                             candidate_digest
                         ) VALUES (
                             $1::oid, $2::uuid, $3::uuid, $4::bigint,
                             $5::bytea, $6::bytea, $7::bytea,
                             $8::bytea, $9::bytea,
                             $10::bytea, $11::bytea,
                             $12::bytea, $13::bytea,
                             $14::bytea, $15::bytea, $16::bytea,
                             $17::bytea
                         )"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        build_id.into(),
                        epoch.into(),
                        candidate.registration_digest.to_vec().into(),
                        candidate.build_spec.clone().into(),
                        candidate.build_spec_digest.to_vec().into(),
                        candidate.generation_descriptor.clone().into(),
                        candidate.generation_descriptor_digest.to_vec().into(),
                        candidate.source_snapshot.clone().into(),
                        candidate.source_snapshot_digest.to_vec().into(),
                        candidate.ready_receipt_set.clone().into(),
                        candidate.ready_receipt_set_digest.to_vec().into(),
                        candidate.epoch_manifest.clone().into(),
                        candidate.manifest_digest.to_vec().into(),
                        candidate.epoch_fingerprint.to_vec().into(),
                        candidate_digest.to_vec().into(),
                    ],
                )
                .map_err(|_| "EC_BUILD_INCOMPLETE: build candidate insert failed".to_owned())?;
            client
                .update(
                    &format!(
                        "UPDATE {registration} SET state = 'Ready'
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid AND state = 'Registered'"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|_| "EC_BUILD_INCOMPLETE: registration Ready transition failed".to_owned())?;
            Ok(())
        })?;

        Ok(candidate_digest.to_vec())
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"))
}

/// Idempotently abort an unpublished coordinator build: abort every
/// participant's unpublished generation, clear the durable build gate by moving
/// the registration to `Aborted`, and release the caller's session locks after
/// the gate-clearing transaction commits (FR-078:354-360). A Decided or
/// Published build cannot be aborted; an already-Aborted or absent build id is a
/// no-op. Remote participants are not yet driven — a multi-node roster fails
/// closed rather than silently skipping a remote generation.
#[pg_extern(volatile, strict)]
fn ec_distann_abort_epoch_build(index_regclass: PgRelation, build_id: Uuid) {
    (|| -> Result<(), String> {
        if !is_rfc4122_v4_uuid(build_id.as_bytes()) {
            return Err("EC_BUILD_ID_CONFLICT: build id must be an RFC 4122 v4 UUID".to_owned());
        }
        let index_oid = index_regclass.oid();
        let (_control_guard, _handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_abort_epoch_build",
        )?;
        let registration = extension_relation_name("ec_distann_build_registration")?;
        let binding = extension_relation_name("ec_distann_build_participant_binding")?;

        // Lock the registration row and read its state. Absence is idempotent.
        let state = Spi::connect_mut(|client| -> Result<Option<String>, String> {
            client
                .update(
                    &format!(
                        "SELECT state FROM {registration}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid
                          FOR UPDATE"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|_| "EC_BUILD_STATE: registration lookup failed".to_owned())?
                .map(|row| {
                    row["state"]
                        .value::<String>()
                        .map_err(|_| "EC_BUILD_STATE: registration state decode failed".to_owned())?
                        .ok_or_else(|| "EC_BUILD_STATE: registration state is NULL".to_owned())
                })
                .next()
                .transpose()
        })?;
        let Some(state) = state else {
            return Ok(());
        };
        match state.as_str() {
            "Aborted" => return Ok(()),
            "Registered" | "Building" | "Ready" => {}
            other => {
                return Err(format!(
                    "EC_BUILD_STATE: cannot abort a build in state {other}"
                ));
            }
        }

        // Abort each participant's unpublished generation. A single-node roster
        // has exactly one local participant (the coordinator); remote dispatch
        // is a later slice.
        let bindings = Spi::connect(|client| -> Result<Vec<bool>, String> {
            client
                .select(
                    &format!(
                        "SELECT is_local FROM {binding}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid
                          ORDER BY roster_ordinal"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|_| "EC_BUILD_STATE: participant binding lookup failed".to_owned())?
                .map(|row| {
                    row["is_local"]
                        .value::<bool>()
                        .map_err(|_| "EC_BUILD_STATE: participant locality decode failed".to_owned())?
                        .ok_or_else(|| "EC_BUILD_STATE: participant locality is NULL".to_owned())
                })
                .collect::<Result<Vec<_>, _>>()
        })?;
        if bindings.iter().any(|is_local| !is_local) {
            return Err(
                "EC_BUILD_STATE: remote participant abort is not yet implemented".to_owned(),
            );
        }
        let abort_handoff = extension_relation_name("ec_distann_abort_epoch_handoff")?;
        for _local in bindings.iter().filter(|is_local| **is_local) {
            Spi::connect_mut(|client| -> Result<(), String> {
                client
                    .update(
                        &format!("SELECT {abort_handoff}($1::oid::regclass, $2::uuid)"),
                        None,
                        &[index_oid.into(), build_id.into()],
                    )
                    .map_err(|_| {
                        "EC_BUILD_STATE: local participant generation abort failed".to_owned()
                    })?;
                Ok(())
            })?;
        }

        // Move the registration to Aborted. The durable gate only matches
        // Registered/Building/Ready/Decided, so this clears it.
        Spi::connect_mut(|client| -> Result<(), String> {
            client
                .update(
                    &format!(
                        "UPDATE {registration} SET state = 'Aborted'
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|_| "EC_BUILD_STATE: registration abort update failed".to_owned())?;
            Ok(())
        })?;

        // Release the coordinator's held session locks, but only after this
        // gate-clearing transaction commits.
        schedule_session_lock_release_for_control(index_oid);
        Ok(())
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"))
}

/// Report the coordinator's view of a build (FR-078:80-83), one row per
/// registered roster participant. The registration supplies the epoch and build
/// state; the publish decision (if any) supplies the decision state; each local
/// participant reports its live generation state, sequence, record count, and a
/// content digest of its Ready receipt. An absent build id yields no rows. A
/// multi-node roster fails closed — remote participant status is a later slice.
#[pg_extern(stable, strict, parallel_restricted)]
#[allow(clippy::type_complexity)]
fn ec_distann_epoch_build_status(
    index_regclass: PgRelation,
    build_id: Uuid,
) -> TableIterator<
    'static,
    (
        name!(epoch, i64),
        name!(build_state, String),
        name!(publish_decision_state, Option<String>),
        name!(node_id, i32),
        name!(participant_state, Option<String>),
        name!(next_batch_seq, Option<i64>),
        name!(record_count, Option<i64>),
        name!(receipt_digest, Option<Vec<u8>>),
        name!(last_error_category, Option<String>),
    ),
> {
    type StatusRow = (
        i64,
        String,
        Option<String>,
        i32,
        Option<String>,
        Option<i64>,
        Option<i64>,
        Option<Vec<u8>>,
        Option<String>,
    );
    let rows = (|| -> Result<Vec<StatusRow>, String> {
        if !is_rfc4122_v4_uuid(build_id.as_bytes()) {
            return Err("EC_BUILD_ID_CONFLICT: build id must be an RFC 4122 v4 UUID".to_owned());
        }
        let index_oid = index_regclass.oid();
        let (_control_guard, _handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            "ec_distann_epoch_build_status",
        )?;
        let registration = extension_relation_name("ec_distann_build_registration")?;
        let decision = extension_relation_name("ec_distann_publish_decision")?;
        let binding = extension_relation_name("ec_distann_build_participant_binding")?;

        // Registration epoch + state. An absent build id yields no rows.
        let registration_row = Spi::connect(|client| -> Result<Option<(i64, String)>, String> {
            client
                .select(
                    &format!(
                        "SELECT epoch, state FROM {registration}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|_| "EC_BUILD_STATE: registration lookup failed".to_owned())?
                .map(|row| {
                    let epoch = row["epoch"]
                        .value::<i64>()
                        .map_err(|_| "EC_BUILD_STATE: registration epoch decode failed".to_owned())?
                        .ok_or_else(|| "EC_BUILD_STATE: registration epoch is NULL".to_owned())?;
                    let state = row["state"]
                        .value::<String>()
                        .map_err(|_| "EC_BUILD_STATE: registration state decode failed".to_owned())?
                        .ok_or_else(|| "EC_BUILD_STATE: registration state is NULL".to_owned())?;
                    Ok((epoch, state))
                })
                .next()
                .transpose()
        })?;
        let Some((epoch, build_state)) = registration_row else {
            return Ok(Vec::new());
        };

        // Publish decision state, if a decision exists for this build.
        let publish_decision_state = Spi::connect(|client| -> Result<Option<String>, String> {
            client
                .select(
                    &format!(
                        "SELECT decision_state FROM {decision}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|_| "EC_BUILD_STATE: decision lookup failed".to_owned())?
                .map(|row| {
                    row["decision_state"]
                        .value::<String>()
                        .map_err(|_| "EC_BUILD_STATE: decision state decode failed".to_owned())?
                        .ok_or_else(|| "EC_BUILD_STATE: decision state is NULL".to_owned())
                })
                .next()
                .transpose()
        })?;

        // Roster participants (node ids), rejecting a not-yet-supported remote roster.
        let bindings = Spi::connect(|client| -> Result<Vec<(i32, bool)>, String> {
            client
                .select(
                    &format!(
                        "SELECT node_id, is_local FROM {binding}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid
                          ORDER BY roster_ordinal"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|_| "EC_BUILD_STATE: participant binding lookup failed".to_owned())?
                .map(|row| {
                    let node_id = row["node_id"]
                        .value::<i32>()
                        .map_err(|_| "EC_BUILD_STATE: participant node id decode failed".to_owned())?
                        .ok_or_else(|| "EC_BUILD_STATE: participant node id is NULL".to_owned())?;
                    let is_local = row["is_local"]
                        .value::<bool>()
                        .map_err(|_| "EC_BUILD_STATE: participant locality decode failed".to_owned())?
                        .ok_or_else(|| "EC_BUILD_STATE: participant locality is NULL".to_owned())?;
                    Ok((node_id, is_local))
                })
                .collect::<Result<Vec<_>, _>>()
        })?;
        if bindings.iter().any(|(_, is_local)| !is_local) {
            return Err(
                "EC_BUILD_STATE: remote participant status is not yet implemented".to_owned(),
            );
        }

        // The single local participant serves the coordinator's own index.
        let generation =
            super::generation_catalog::lookup_generation(index_oid, logical_index_uuid, build_id)?;

        let mut rows = Vec::with_capacity(bindings.len());
        for (node_id, _is_local) in bindings {
            let matching = generation
                .as_ref()
                .filter(|row| i32::try_from(row.node_id).ok() == Some(node_id));
            let (participant_state, next_batch_seq, record_count, receipt_digest) = match matching {
                Some(row) => (
                    Some(row.state.clone()),
                    Some(i64::try_from(row.next_batch_seq).map_err(|_| {
                        "EC_BUILD_STATE: next batch sequence exceeds bigint".to_owned()
                    })?),
                    Some(i64::try_from(row.cumulative_record_count).map_err(|_| {
                        "EC_BUILD_STATE: record count exceeds bigint".to_owned()
                    })?),
                    row.ready_receipt
                        .map(|receipt| domain_digest(b"ec_distann_ready_receipt_v1\0", &receipt).to_vec()),
                ),
                None => (None, None, None, None),
            };
            rows.push((
                epoch,
                build_state.clone(),
                publish_decision_state.clone(),
                node_id,
                participant_state,
                next_batch_seq,
                record_count,
                receipt_digest,
                None,
            ));
        }
        Ok(rows)
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"));
    TableIterator::new(rows.into_iter())
}

#[cfg(test)]
mod tests {
    use super::super::canonical_wire::sample_rfc4122_v4_uuid;
    use super::*;

    fn participant(secret: &str) -> DesiredParticipant {
        DesiredParticipant {
            roster_ordinal: 0,
            node_id: 17,
            endpoint_identity: "registration/node-17".to_owned(),
            conninfo_secret_name: secret.to_owned(),
            remote_index_regclass: "public.registration_idx".to_owned(),
            participant_logical_index_uuid: Uuid::from_bytes(sample_rfc4122_v4_uuid(0x17)),
            compatibility_digest: [0x22; 32],
            is_local: true,
        }
    }

    #[test]
    fn registration_digest_golden_binds_private_transport_fields() {
        let participants = vec![participant("REGISTRATION_SECRET")];
        let roster = public_roster(&participants).unwrap();
        let snapshot = encode_roster_snapshot(&roster).unwrap();
        let digest = registration_digest(
            pg_sys::Oid::from(1234_u32),
            Uuid::from_bytes(sample_rfc4122_v4_uuid(0xA1)),
            pg_sys::Oid::from(5678_u32),
            7,
            Uuid::from_bytes(sample_rfc4122_v4_uuid(0xAB)),
            9,
            &snapshot,
            roster_digest(&roster).unwrap(),
            [0x11; 32],
            [0x22; 32],
            &participants,
        )
        .unwrap();
        assert_eq!(
            hex::encode(digest),
            "c5a90122402eb68d6f443d63fe3e5744c07ff902a27e02d02125494c290f25ab"
        );

        let changed = vec![participant("REGISTRATION_CHANGED")];
        assert_ne!(
            digest,
            registration_digest(
                pg_sys::Oid::from(1234_u32),
                Uuid::from_bytes(sample_rfc4122_v4_uuid(0xA1)),
                pg_sys::Oid::from(5678_u32),
                7,
                Uuid::from_bytes(sample_rfc4122_v4_uuid(0xAB)),
                9,
                &snapshot,
                roster_digest(&roster).unwrap(),
                [0x11; 32],
                [0x22; 32],
                &changed,
            )
            .unwrap()
        );
    }
}
