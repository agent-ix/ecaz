//! Coordinator build registration and durable private participant bindings.

mod cancel;
mod t1;
mod t2;
mod t3;
mod t4a;

pub(crate) use t2::load_build_candidate;

use std::cell::{Cell, RefCell};

use pgrx::datum::Uuid;
use pgrx::iter::TableIterator;
use pgrx::{name, pg_extern, pg_sys, PgRelation, Spi};

use crate::storage::relation::index_heap_relation_oid_handle;

use super::canonical_wire::{domain_digest, fixed_digest, is_rfc4122_v4_uuid, CanonicalEncoder};
use super::generation_catalog::extension_relation_name;
use super::generation_descriptor::{
    validate_roster, DistannBuildOptions, DistannBuildSpec, DistannGenerationDescriptor,
    DistannHeadPolicy, DistannRosterEntry,
};
use super::generation_store::{control_compatibility_digest, open_control_index};
use super::handoff_router::{DistannHandoffRouteIdentity, DistannStageAck};
use super::lifecycle_state::{
    require_exact_transition, PredecessorDisposition, PublishDecisionState, RegistrationState,
};
use super::lifecycle_wire::{
    DistannBuildCandidateV1, DistannCancelPublishAuditV1, DistannPublishedEpochIdentity,
    DistannSuccessorActivationV1,
};
use super::manifest_v2::{
    DistannEpochManifestV2, DistannManifestBuildOptions, DistannManifestCodecParameters,
    DistannSourceSnapshot,
};
use super::roster_digest;
use super::row_schema::resolve_relation_schema;

const BUILD_REGISTRATION_V1_VERSION: u16 = 1;
const BUILD_REGISTRATION_V2_VERSION: u16 = 2;
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
        let owns_source_transaction_lock = unsafe {
            pg_sys::ConditionalLockRelationOid(
                source_relation_oid,
                pg_sys::ShareLock as pg_sys::LOCKMODE,
            )
        };
        if !owns_source_transaction_lock {
            return Err(
                "EC_BUILD_BUSY: source relation is busy with a conflicting operation".to_owned(),
            );
        }
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

    fn reacquired_after_owner_loss(&self) -> bool {
        self.acquired
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

fn preflight_build_lock_identity(
    index_oid: pg_sys::Oid,
    context: &'static str,
) -> Result<(pg_sys::Oid, Uuid), String> {
    let (control, handle, _metadata, logical_index_uuid) = open_control_index(
        index_oid,
        pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        context,
    )?;
    let source_relation_oid = index_heap_relation_oid_handle(handle);
    drop(control);
    Ok((source_relation_oid, logical_index_uuid))
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
            .map_err(|error| format!("EC_BUILD_STATE: registry revision lock failed: {error}"))?
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
            .map_err(|error| format!("EC_BUILD_STATE: desired roster lookup failed: {error}"))?
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
                        "EC_BUILD_STATE",
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
            .map_err(|error| format!("EC_BUILD_STATE: build slot lookup failed: {error}"))?
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
    payload_cover_descriptor_digest: Option<[u8; 32]>,
    compatibility_digest: [u8; 32],
    participants: &[DesiredParticipant],
) -> Result<Vec<u8>, String> {
    let mut encoder = CanonicalEncoder::with_capacity(
        192 + roster_snapshot.len() + participants.len().saturating_mul(160),
    );
    encoder.put_u16(if payload_cover_descriptor_digest.is_some() {
        BUILD_REGISTRATION_V2_VERSION
    } else {
        BUILD_REGISTRATION_V1_VERSION
    });
    encoder.put_u32(u32::from(index_oid));
    encoder.put_fixed(logical_index_uuid.as_bytes());
    encoder.put_u32(u32::from(source_relation_oid));
    encoder.put_u64(epoch);
    encoder.put_fixed(build_id.as_bytes());
    encoder.put_u64(registry_revision);
    encoder.put_len_prefixed(roster_snapshot)?;
    encoder.put_fixed(&roster_digest);
    encoder.put_fixed(&row_schema_fingerprint);
    if let Some(digest) = payload_cover_descriptor_digest {
        encoder.put_fixed(&digest);
    }
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
    payload_cover_descriptor_digest: Option<[u8; 32]>,
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
            payload_cover_descriptor_digest,
            compatibility_digest,
            participants,
        )?,
    ))
}

#[derive(Debug)]
struct StoredRegistration {
    source_relation_oid: pg_sys::Oid,
    epoch: u64,
    state: RegistrationState,
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
    current_payload_cover_descriptor_digest: Option<[u8; 32]>,
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
            .map_err(|error| format!("EC_BUILD_STATE: registration replay lookup failed: {error}"))?
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
                    state: RegistrationState::parse(
                        &row["state"]
                            .value::<String>()
                            .map_err(|error| {
                                format!("EC_BUILD_STATE: registration state decode failed: {error}")
                            })?
                            .ok_or_else(|| {
                                "EC_BUILD_STATE: registration state is NULL".to_owned()
                            })?,
                    )?,
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
                        "EC_BUILD_STATE",
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
                        "EC_BUILD_STATE",
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
                        "EC_BUILD_STATE",
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
    if stored.state == RegistrationState::Aborted {
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
            .map_err(|error| {
                format!("EC_BUILD_STATE: participant binding replay lookup failed: {error}")
            })?
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
                        "EC_BUILD_STATE",
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
        current_payload_cover_descriptor_digest,
        current_compatibility_digest,
        &participants,
    )?;
    if expected_digest != stored.registration_digest {
        return Err("EC_BUILD_STATE: durable registration digest is inconsistent".to_owned());
    }
    Ok(Some((
        stored.registration_digest,
        stored.state != RegistrationState::Published,
    )))
}

fn recover_predecessor_retirement(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    successor_build_id: Uuid,
    activation_bytes: &[u8],
    activation_digest: &[u8],
) -> Result<(), String> {
    let disposition = extension_relation_name("ec_distann_predecessor_disposition")?;
    let binding = extension_relation_name("ec_distann_build_participant_binding")?;
    let pending = Spi::connect_mut(
        |client| -> Result<Vec<(i32, bool, String, String)>, String> {
            client
                .update(
                    &format!(
                        "SELECT p.predecessor_roster_ordinal, b.is_local,
                            b.conninfo_secret_name, b.remote_index_regclass
                       FROM {disposition} p
                       JOIN {binding} b
                         ON b.index_oid = p.index_oid
                        AND b.logical_index_uuid = p.logical_index_uuid
                        AND b.build_id = p.predecessor_build_id
                        AND b.roster_ordinal = p.predecessor_roster_ordinal
                      WHERE p.index_oid = $1::oid
                        AND p.logical_index_uuid = $2::uuid
                        AND p.successor_build_id = $3::uuid
                        AND p.disposition = 'Pending'
                      ORDER BY p.predecessor_roster_ordinal
                      FOR UPDATE OF p"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        successor_build_id.into(),
                    ],
                )
                .map_err(|error| {
                    format!("EC_PREDECESSOR_RETIRE_PENDING: disposition lookup failed: {error}")
                })?
                .map(|row| {
                    let ordinal = row["predecessor_roster_ordinal"]
                        .value::<i32>()
                        .map_err(|_| {
                            "EC_PREDECESSOR_RETIRE_PENDING: ordinal decode failed".to_owned()
                        })?
                        .ok_or_else(|| {
                            "EC_PREDECESSOR_RETIRE_PENDING: ordinal is NULL".to_owned()
                        })?;
                    let is_local = row["is_local"]
                        .value::<bool>()
                        .map_err(|_| {
                            "EC_PREDECESSOR_RETIRE_PENDING: locality decode failed".to_owned()
                        })?
                        .ok_or_else(|| {
                            "EC_PREDECESSOR_RETIRE_PENDING: locality is NULL".to_owned()
                        })?;
                    let text = |field: &str| -> Result<String, String> {
                        row[field]
                            .value::<String>()
                            .map_err(|_| {
                                format!("EC_PREDECESSOR_RETIRE_PENDING: {field} decode failed")
                            })?
                            .ok_or_else(|| {
                                format!("EC_PREDECESSOR_RETIRE_PENDING: {field} is NULL")
                            })
                    };
                    Ok((
                        ordinal,
                        is_local,
                        text("conninfo_secret_name")?,
                        text("remote_index_regclass")?,
                    ))
                })
                .collect()
        },
    )?;
    let mark = extension_relation_name("ec_distann_mark_epoch_retired")?;
    for (ordinal, is_local, secret_name, remote_index_regclass) in &pending {
        if *is_local {
            Spi::connect_mut(|client| -> Result<(), String> {
                client
                    .update(
                        &format!("SELECT {mark}($1::oid::regclass, $2::bytea, $3::bytea)"),
                        None,
                        &[
                            index_oid.into(),
                            activation_bytes.to_vec().into(),
                            activation_digest.to_vec().into(),
                        ],
                    )
                    .map_err(|error| {
                        format!("EC_PREDECESSOR_RETIRE_PENDING: local mark failed: {error}")
                    })?;
                Ok(())
            })?;
        } else {
            let conninfo = super::node_registry::resolve_conninfo_secret(secret_name)?;
            super::remote_transport::remote_mark_epoch_retired(
                conninfo.conninfo(),
                remote_index_regclass,
                activation_bytes,
                activation_digest,
            )?;
        }
        let updated = Spi::connect_mut(|client| {
            client
                .update(
                    &format!(
                        "UPDATE {disposition}
                            SET disposition = 'Retired',
                                retired_activation_digest = $4::bytea,
                                updated_at = clock_timestamp()
                          WHERE index_oid = $1::oid
                            AND logical_index_uuid = $2::uuid
                            AND successor_build_id = $3::uuid
                            AND predecessor_roster_ordinal = $5::integer
                            AND disposition = 'Pending'
                          RETURNING 1"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        successor_build_id.into(),
                        activation_digest.to_vec().into(),
                        (*ordinal).into(),
                    ],
                )
                .map(|rows| rows.len())
                .map_err(|error| {
                    format!("EC_PREDECESSOR_RETIRE_PENDING: disposition update failed: {error}")
                })
        })?;
        require_exact_transition(
            PredecessorDisposition::Pending,
            PredecessorDisposition::Retired,
            updated,
            "predecessor disposition",
        )?;
    }

    let decision = extension_relation_name("ec_distann_publish_decision")?;
    let applied = Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "UPDATE {decision} d
                        SET decision_state = 'Applied', applied_at = clock_timestamp()
                      WHERE d.index_oid = $1::oid
                        AND d.logical_index_uuid = $2::uuid
                        AND d.build_id = $3::uuid
                        AND d.decision_state = 'Activated'
                        AND NOT EXISTS (
                            SELECT 1 FROM {disposition} p
                             WHERE p.index_oid = d.index_oid
                               AND p.logical_index_uuid = d.logical_index_uuid
                               AND p.successor_build_id = d.build_id
                               AND p.disposition = 'Pending'
                        )
                      RETURNING 1"
                ),
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    successor_build_id.into(),
                ],
            )
            .map(|rows| rows.len())
            .map_err(|error| {
                format!("EC_PREDECESSOR_RETIRE_PENDING: covering decision apply failed: {error}")
            })
    })?;
    require_exact_transition(
        PublishDecisionState::Activated,
        PublishDecisionState::Applied,
        applied,
        "publish decision",
    )
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
                .map_err(|error| format!("EC_BUILD_STATE: registration lookup failed: {error}"))?
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
                .map_err(|error| format!("EC_BUILD_STATE: decision lookup failed: {error}"))?
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
                .map_err(|error| {
                    format!("EC_BUILD_STATE: participant binding lookup failed: {error}")
                })?
                .map(|row| {
                    let node_id = row["node_id"]
                        .value::<i32>()
                        .map_err(|_| {
                            "EC_BUILD_STATE: participant node id decode failed".to_owned()
                        })?
                        .ok_or_else(|| "EC_BUILD_STATE: participant node id is NULL".to_owned())?;
                    let is_local = row["is_local"]
                        .value::<bool>()
                        .map_err(|_| {
                            "EC_BUILD_STATE: participant locality decode failed".to_owned()
                        })?
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
            let (participant_state, next_batch_seq, record_count, receipt_digest) =
                match matching {
                    Some(row) => (
                        Some(row.state.to_string()),
                        Some(i64::try_from(row.next_batch_seq).map_err(|_| {
                            "EC_BUILD_STATE: next batch sequence exceeds bigint".to_owned()
                        })?),
                        Some(i64::try_from(row.cumulative_record_count).map_err(|_| {
                            "EC_BUILD_STATE: record count exceeds bigint".to_owned()
                        })?),
                        row.ready_receipt.as_ref().map(|receipt| {
                            domain_digest(b"ec_distann_ready_receipt_v1\0", &receipt).to_vec()
                        }),
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
            None,
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
                None,
                [0x22; 32],
                &changed,
            )
            .unwrap()
        );

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
                Some([0x33; 32]),
                [0x22; 32],
                &participants,
            )
            .unwrap()
        );
    }
}
