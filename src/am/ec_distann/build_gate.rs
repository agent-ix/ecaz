//! Durable coordinator build-gate enforcement.
//!
//! Session relation locks are the fast path while the build coordinator is
//! alive. PostgreSQL releases those locks when that backend exits or aborts a
//! later top-level transaction, so the durable registration remains the
//! authority. These hooks reject source rewrites and control-index identity
//! changes until explicit abort or publication recovery clears the gate.

use std::cell::Cell;
use std::ffi::CStr;
use std::panic::AssertUnwindSafe;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};

use pgrx::datum::Uuid;
use pgrx::prelude::{AllocatedByPostgres, PgHeapTuple, PgTrigger, PgTriggerError};
use pgrx::{pg_extern, pg_guard, pg_sys, FromDatum, Spi};

const SOURCE_GATE: i32 = 1;
const CONTROL_GATE: i32 = 2;

static HOOKS_INSTALLED: AtomicBool = AtomicBool::new(false);
static LOADED_DURING_SHARED_PRELOAD: AtomicBool = AtomicBool::new(false);
static mut PREVIOUS_EXECUTOR_START_HOOK: pg_sys::ExecutorStart_hook_type = None;
static mut PREVIOUS_PROCESS_UTILITY_HOOK: pg_sys::ProcessUtility_hook_type = None;

thread_local! {
    /// The catalog helper plans SQL of its own. Suppress recursive gate checks
    /// while resolving/calling it, but never suppress the next installed hook.
    static INSIDE_GATE_LOOKUP: Cell<bool> = const { Cell::new(false) };
    static GATE_FUNCTION_OID: Cell<pg_sys::Oid> = const { Cell::new(pg_sys::InvalidOid) };
    /// Backend-local steady-state fast path. The registration-table trigger
    /// publishes a transactional relcache invalidation on every mutation, so
    /// a committed first/last active gate clears this bit in every backend.
    static NO_ACTIVE_GATE: Cell<bool> = const { Cell::new(false) };
    static GLOBAL_UTILITY_LOCK_DEPTH: Cell<u32> = const { Cell::new(0) };
}

const GLOBAL_GATE_ADVISORY_KEY: i64 = 0x4543_445A_4741_5445;

struct LookupGuard;

impl LookupGuard {
    fn enter() -> Option<Self> {
        INSIDE_GATE_LOOKUP.with(|inside| {
            if inside.replace(true) {
                None
            } else {
                Some(Self)
            }
        })
    }
}

impl Drop for LookupGuard {
    fn drop(&mut self) {
        INSIDE_GATE_LOOKUP.with(|inside| inside.set(false));
    }
}

/// Return a bit mask describing whether `relation_oid` is the source (bit 0)
/// or coordinator control index (bit 1) of a live durable build gate.
///
/// This helper re-opens the current control index and compares the UUID stored
/// in its v5 metadata. Consequently an unrelated relation that reuses a stale
/// OID is not gated by catalog coincidence alone.
#[pg_extern(stable, strict, parallel_restricted)]
fn ec_distann_build_gate_relation_mask(relation_oid: pg_sys::Oid) -> i32 {
    let registration =
        super::generation_catalog::extension_relation_name("ec_distann_build_registration")
            .unwrap_or_else(|error| pgrx::error!("{error}"));
    let rows = Spi::connect(|client| -> Result<Vec<_>, String> {
        client
            .select(
                &format!(
                    "SELECT index_oid, logical_index_uuid, source_relid
                       FROM {registration}
                      WHERE state IN ('Registered', 'Building', 'Ready', 'Decided')
                        AND ($1::bigint = 0
                             OR source_relid = $1::bigint::oid
                             OR index_oid = $1::bigint::oid)"
                ),
                None,
                &[i64::from(relation_oid.to_u32()).into()],
            )
            .unwrap_or_else(|_| pgrx::error!("EC_BUILD_STATE: durable build gate lookup failed"))
            .map(|row| -> Result<_, String> {
                let required_oid = |name: &str| -> Result<pg_sys::Oid, String> {
                    row[name]
                        .value::<pg_sys::Oid>()
                        .map_err(|_| format!("EC_BUILD_STATE: durable gate {name} decode failed"))?
                        .ok_or_else(|| format!("EC_BUILD_STATE: durable gate {name} is NULL"))
                };
                let logical_uuid = row["logical_index_uuid"]
                    .value::<Uuid>()
                    .map_err(|_| "EC_BUILD_STATE: durable gate UUID decode failed".to_owned())?
                    .ok_or_else(|| "EC_BUILD_STATE: durable gate UUID is NULL".to_owned())?;
                Ok((
                    required_oid("index_oid")?,
                    logical_uuid,
                    required_oid("source_relid")?,
                ))
            })
            .collect::<Result<Vec<_>, _>>()
    })
    .unwrap_or_else(|error| pgrx::error!("{error}"));

    let mut mask = 0;
    for (index_oid, expected_uuid, source_oid) in rows {
        let control = super::generation_store::open_control_index(
            index_oid,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            "ec_distann durable build gate",
        );
        let (_guard, _handle, _metadata, live_uuid) = match control {
            Ok(control) => control,
            Err(error) => {
                let distann_am = unsafe { pg_sys::get_am_oid(c"ec_distann".as_ptr(), false) };
                let live_am = unsafe { pg_sys::get_rel_relam(index_oid) };
                if live_am == pg_sys::InvalidOid || live_am != distann_am {
                    continue;
                }
                pgrx::error!("EC_BUILD_STATE: durable gate control validation failed: {error}");
            }
        };
        if live_uuid != expected_uuid {
            continue;
        }
        let live_source = unsafe { pg_sys::IndexGetRelation(index_oid, true) };
        if live_source == pg_sys::InvalidOid || live_source != source_oid {
            continue;
        }
        if relation_oid == pg_sys::InvalidOid || relation_oid == source_oid {
            mask |= SOURCE_GATE;
        }
        if relation_oid == index_oid {
            mask |= CONTROL_GATE;
        }
    }
    mask
}

fn internal_gate_function_oid() -> Result<pg_sys::Oid, String> {
    let cached = GATE_FUNCTION_OID.with(Cell::get);
    if cached != pg_sys::InvalidOid {
        return Ok(cached);
    }
    let qualified =
        super::generation_catalog::extension_relation_name("ec_distann_build_gate_relation_mask")?;
    let signature = format!("{qualified}(oid)");
    let resolved = Spi::connect(|client| {
        client
            .select(
                "SELECT $1::text::regprocedure::oid AS function_oid",
                None,
                &[signature.into()],
            )
            .map_err(|_| "EC_BUILD_STATE: durable build gate helper lookup failed".to_owned())?
            .map(|row| {
                row["function_oid"]
                    .value::<pg_sys::Oid>()
                    .map_err(|_| {
                        "EC_BUILD_STATE: durable build gate helper decode failed".to_owned()
                    })?
                    .ok_or_else(|| "EC_BUILD_STATE: durable build gate helper is NULL".to_owned())
            })
            .next()
            .transpose()?
            .ok_or_else(|| "EC_BUILD_STATE: durable build gate helper is missing".to_owned())
    })?;
    GATE_FUNCTION_OID.with(|cached| cached.set(resolved));
    Ok(resolved)
}

#[pg_guard]
unsafe extern "C-unwind" fn invalidate_gate_function_oid(
    _arg: pg_sys::Datum,
    _cache_id: core::ffi::c_int,
    _hash_value: u32,
) {
    GATE_FUNCTION_OID.with(|cached| cached.set(pg_sys::InvalidOid));
}

#[pg_guard]
unsafe extern "C-unwind" fn invalidate_no_active_gate_cache(
    _arg: pg_sys::Datum,
    _relation_oid: pg_sys::Oid,
) {
    // Clearing on every relcache event is deliberately conservative. DDL is
    // much rarer than DML, and this avoids retaining a relation OID across
    // DROP EXTENSION / CREATE EXTENSION cycles.
    NO_ACTIVE_GATE.with(|cached| cached.set(false));
    super::traversal_replica::invalidate_ready_replica_presence_cache();
}

/// Publish registration-table mutations through PostgreSQL's transactional
/// relcache invalidation channel. The local cache is cleared immediately for
/// same-transaction visibility; other backends receive the invalidation when
/// the mutating transaction commits.
#[pgrx::pg_trigger]
fn ec_distann_build_gate_registration_changed<'a>(
    trigger: &'a PgTrigger<'a>,
) -> Result<Option<PgHeapTuple<'a, AllocatedByPostgres>>, PgTriggerError> {
    let relation_oid = trigger.relid()?;
    NO_ACTIVE_GATE.with(|cached| cached.set(false));
    unsafe { pg_sys::CacheInvalidateRelcacheByRelid(relation_oid) };
    Ok(None)
}

fn extension_is_installed() -> bool {
    unsafe { pg_sys::get_extension_oid(c"ecaz".as_ptr(), true) != pg_sys::InvalidOid }
}

pub(crate) fn require_shared_preload() -> Result<(), String> {
    if LOADED_DURING_SHARED_PRELOAD.load(Ordering::Acquire) {
        Ok(())
    } else {
        Err("EC_EPOCH_REGISTRY_UNAVAILABLE: distributed builds require ecaz in shared_preload_libraries and a PostgreSQL restart".to_owned())
    }
}

/// Serialize commands whose affected relation set is discovered only inside
/// standard_ProcessUtility with creation of a new durable registration.
/// Begin-build takes the shared side before its source lock; global rewrite
/// utilities take the exclusive side before inspecting active registrations.
pub(crate) fn lock_global_gate_serialization(exclusive: bool) -> Result<(), String> {
    let function_oid = if exclusive {
        pg_sys::Oid::from(pg_sys::F_PG_ADVISORY_LOCK_INT8)
    } else {
        pg_sys::Oid::from(pg_sys::F_PG_ADVISORY_XACT_LOCK_SHARED_INT8)
    };
    unsafe {
        pg_sys::OidFunctionCall1Coll(
            function_oid,
            pg_sys::InvalidOid,
            pg_sys::Datum::from(GLOBAL_GATE_ADVISORY_KEY),
        );
    }
    if exclusive {
        GLOBAL_UTILITY_LOCK_DEPTH.with(|depth| depth.set(depth.get().saturating_add(1)));
    }
    Ok(())
}

fn release_global_utility_lock_if_acquired(previous_depth: u32) {
    let held = GLOBAL_UTILITY_LOCK_DEPTH.with(|depth| {
        let current = depth.get();
        if current > previous_depth {
            depth.set(current - 1);
            true
        } else {
            false
        }
    });
    if !held {
        return;
    }
    unsafe {
        pg_sys::OidFunctionCall1Coll(
            pg_sys::Oid::from(pg_sys::F_PG_ADVISORY_UNLOCK_INT8),
            pg_sys::InvalidOid,
            pg_sys::Datum::from(GLOBAL_GATE_ADVISORY_KEY),
        );
    }
}

fn invoke_gate_mask_uncached(relation_oid: pg_sys::Oid) -> i32 {
    let Some(_guard) = LookupGuard::enter() else {
        return 0;
    };
    let function_oid = internal_gate_function_oid().unwrap_or_else(|error| pgrx::error!("{error}"));
    let datum = unsafe {
        pg_sys::OidFunctionCall1Coll(
            function_oid,
            pg_sys::InvalidOid,
            pg_sys::Datum::from(relation_oid),
        )
    };
    unsafe { i32::from_datum(datum, false) }
        .unwrap_or_else(|| pgrx::error!("EC_BUILD_STATE: durable build gate helper returned NULL"))
}

fn invoke_gate_mask(relation_oid: pg_sys::Oid) -> i32 {
    // A shared-preloaded library is present in every database, including
    // databases where CREATE EXTENSION has not run (or has been dropped).
    // There cannot be a durable gate without the extension catalogs, and
    // ordinary DML in those databases must remain usable.
    if !extension_is_installed() {
        return 0;
    }
    if NO_ACTIVE_GATE.with(Cell::get) {
        return 0;
    }

    // Establish the database-wide negative before probing a particular result
    // relation. The registration-table statement trigger clears this cache in
    // the mutating backend immediately and broadcasts a relcache invalidation
    // transactionally, so a different backend cannot commit the first durable
    // registration while this negative remains usable. Begin-build also holds
    // the conflicting source relation lock before inserting the registration,
    // closing the statement/invalidation boundary for source DML.
    if relation_oid != pg_sys::InvalidOid {
        let any_mask = invoke_gate_mask_uncached(pg_sys::InvalidOid);
        if any_mask & SOURCE_GATE == 0 {
            NO_ACTIVE_GATE.with(|cached| cached.set(true));
            return 0;
        }
    }

    let mask = invoke_gate_mask_uncached(relation_oid);
    if relation_oid == pg_sys::InvalidOid && mask & SOURCE_GATE == 0 {
        NO_ACTIVE_GATE.with(|cached| cached.set(true));
    }
    mask
}

fn relation_gate_mask(relation_oid: pg_sys::Oid) -> i32 {
    if relation_oid == pg_sys::InvalidOid {
        0
    } else {
        invoke_gate_mask(relation_oid)
    }
}

fn reject_if_any_source_gate(operation: &str) {
    if invoke_gate_mask(pg_sys::InvalidOid) & SOURCE_GATE != 0 {
        pgrx::error!(
            "EC_BUILD_STATE: {operation} is blocked by an active ec_distann distributed build"
        );
    }
}

fn reject_if_gated(relation_oid: pg_sys::Oid, required_mask: i32, operation: &str) {
    if relation_gate_mask(relation_oid) & required_mask != 0 {
        pgrx::error!(
            "EC_BUILD_STATE: {operation} is blocked by an active ec_distann distributed build"
        );
    }
}

fn ec_distann_indexes_for_source(source_oid: pg_sys::Oid) -> Vec<pg_sys::Oid> {
    let Some(_guard) = LookupGuard::enter() else {
        return Vec::new();
    };
    Spi::connect(|client| {
        client
            .select(
                "SELECT index_relation.oid AS index_oid
                   FROM pg_catalog.pg_index index_catalog
                   JOIN pg_catalog.pg_class index_relation
                     ON index_relation.oid = index_catalog.indexrelid
                   JOIN pg_catalog.pg_am access_method
                     ON access_method.oid = index_relation.relam
                  WHERE index_catalog.indrelid = $1::bigint::oid
                    AND index_catalog.indisvalid
                    AND index_catalog.indisready
                    AND access_method.amname = 'ec_distann'
                  ORDER BY index_relation.oid",
                None,
                &[i64::from(source_oid.to_u32()).into()],
            )
            .unwrap_or_else(|_| pgrx::error!("EC_REPLICA_INVALIDATION: source index lookup failed"))
            .map(|row| {
                row["index_oid"]
                    .value::<pg_sys::Oid>()
                    .unwrap_or_else(|_| {
                        pgrx::error!("EC_REPLICA_INVALIDATION: source index OID decode failed")
                    })
                    .unwrap_or_else(|| {
                        pgrx::error!("EC_REPLICA_INVALIDATION: source index OID is NULL")
                    })
            })
            .collect()
    })
}

fn guard_source_traversal_replicas(source_oid: pg_sys::Oid) {
    if !super::traversal_replica::ready_replica_may_exist()
        .unwrap_or_else(|error| pgrx::error!("{error}"))
    {
        return;
    }
    for index_oid in ec_distann_indexes_for_source(source_oid) {
        super::traversal_replica::guard_traversal_replica_mutation(index_oid);
    }
}

/// Guard every result relation of a finished plan. `resultRelations` is the
/// global integer list of RT indexes that any `ModifyTable` node writes,
/// including targets introduced by data-modifying CTEs inside an otherwise
/// read-only `SELECT`, so no `commandType` filter is needed. Pure reads carry
/// an empty list and never invoke either the durable build mask or traversal
/// replica invalidation.
unsafe fn guard_planned_result_relations(plannedstmt: *mut pg_sys::PlannedStmt) {
    let Some(stmt) = (unsafe { plannedstmt.as_ref() }) else {
        return;
    };
    // Utility statements plan no `ModifyTable` node; the ProcessUtility hook
    // owns their enforcement.
    if !stmt.utilityStmt.is_null() {
        return;
    }
    let table_length = unsafe { pg_sys::list_length(stmt.rtable) };
    let count = unsafe { pg_sys::list_length(stmt.resultRelations) };
    for offset in 0..count {
        let rt_index = unsafe { pg_sys::list_nth_int(stmt.resultRelations, offset) };
        if rt_index <= 0 || rt_index > table_length {
            continue;
        }
        let rte =
            unsafe { pg_sys::list_nth(stmt.rtable, rt_index - 1).cast::<pg_sys::RangeTblEntry>() };
        let Some(rte) = (unsafe { rte.as_ref() }) else {
            continue;
        };
        if rte.rtekind == pg_sys::RTEKind::RTE_RELATION && rte.relid != pg_sys::InvalidOid {
            reject_if_gated(rte.relid, SOURCE_GATE, "source DML");
            guard_source_traversal_replicas(rte.relid);
        }
    }
}

fn call_next_executor_start(query_desc: *mut pg_sys::QueryDesc, eflags: core::ffi::c_int) {
    unsafe {
        if let Some(previous) = PREVIOUS_EXECUTOR_START_HOOK {
            previous(query_desc, eflags);
        } else {
            pg_sys::standard_ExecutorStart(query_desc, eflags);
        }
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_distann_build_gate_executor_start_hook(
    query_desc: *mut pg_sys::QueryDesc,
    eflags: core::ffi::c_int,
) {
    // Enforce at executor start rather than plan time so a cached generic plan,
    // a PL/pgSQL cached plan, or a foreign-key trigger's cached SPI plan built
    // before the registration committed still re-enters the gate on execution
    // through `CheckCachedPlan`/`AcquireExecutorLocks`, which never replan.
    //
    // C-language validation loads the library (and invokes `_PG_init`) while
    // CREATE EXTENSION is still installing SQL objects, so installation DML must
    // pass through until PostgreSQL clears `creating_extension`.
    // Logical replication's apply worker can call ExecSimpleRelationInsert /
    // Update / Delete without ExecutorStart. Task 179 does not claim that
    // path: distributed source tables must remain outside publications until
    // a replication-specific gate is added.
    let explain_only = (eflags as u32 & pg_sys::EXEC_FLAG_EXPLAIN_ONLY) != 0;
    if !explain_only
        && !unsafe { pg_sys::creating_extension }
        && !INSIDE_GATE_LOOKUP.with(Cell::get)
    {
        if let Some(desc) = unsafe { query_desc.as_ref() } {
            unsafe { guard_planned_result_relations(desc.plannedstmt) };
        }
    }
    call_next_executor_start(query_desc, eflags);
}

unsafe fn rangevar_oid(rangevar: *mut pg_sys::RangeVar) -> pg_sys::Oid {
    if rangevar.is_null() {
        return pg_sys::InvalidOid;
    }
    // This hook runs before standard_ProcessUtility. Resolve only: PostgreSQL's
    // statement-specific callbacks must perform ownership checks before the
    // statement acquires its true lock level. Acquiring an escalated lock here
    // with a NULL callback would let an unprivileged failing command lock an
    // arbitrary nameable relation and would over-lock CONCURRENTLY variants.
    unsafe {
        pg_sys::RangeVarGetRelidExtended(
            rangevar,
            pg_sys::NoLock as pg_sys::LOCKMODE,
            pg_sys::RVROption::RVR_MISSING_OK,
            None,
            ptr::null_mut(),
        )
    }
}

unsafe fn gate_drop(stmt: *mut pg_sys::DropStmt) {
    let Some(stmt) = (unsafe { stmt.as_ref() }) else {
        return;
    };
    if stmt.behavior == pg_sys::DropBehavior::DROP_CASCADE
        && stmt.removeType == pg_sys::ObjectType::OBJECT_SCHEMA
    {
        lock_global_gate_serialization(true).unwrap_or_else(|error| pgrx::error!("{error}"));
        reject_if_any_source_gate("DROP SCHEMA CASCADE");
        return;
    }
    let required_mask = match stmt.removeType {
        pg_sys::ObjectType::OBJECT_TABLE => SOURCE_GATE,
        pg_sys::ObjectType::OBJECT_INDEX => CONTROL_GATE,
        _ => return,
    };
    let count = unsafe { pg_sys::list_length(stmt.objects) };
    for offset in 0..count {
        let names = unsafe { pg_sys::list_nth(stmt.objects, offset).cast::<pg_sys::List>() };
        if names.is_null() {
            continue;
        }
        let rangevar = unsafe { pg_sys::makeRangeVarFromNameList(names) };
        let oid = unsafe { rangevar_oid(rangevar) };
        reject_if_gated(oid, required_mask, "DROP");
    }
}

unsafe fn vacuum_is_full(stmt: &pg_sys::VacuumStmt) -> bool {
    let count = unsafe { pg_sys::list_length(stmt.options) };
    for offset in 0..count {
        let option = unsafe { pg_sys::list_nth(stmt.options, offset).cast::<pg_sys::DefElem>() };
        let Some(option_ref) = (unsafe { option.as_ref() }) else {
            continue;
        };
        if option_ref.defname.is_null() {
            continue;
        }
        let name = unsafe { CStr::from_ptr(option_ref.defname) }.to_bytes();
        if name == b"full" && unsafe { pg_sys::defGetBoolean(option) } {
            return true;
        }
    }
    false
}

unsafe fn gate_utility_node(node: *mut pg_sys::Node) {
    let Some(node_ref) = (unsafe { node.as_ref() }) else {
        return;
    };
    match node_ref.type_ {
        pg_sys::NodeTag::T_AlterTableStmt => {
            let stmt = unsafe { &*node.cast::<pg_sys::AlterTableStmt>() };
            let oid = unsafe { rangevar_oid(stmt.relation) };
            reject_if_gated(oid, SOURCE_GATE | CONTROL_GATE, "ALTER");

            // ATTACH names the parent in AlterTableStmt::relation and the
            // prospective child in PartitionCmd. A build may have registered
            // the child while it was standalone, so gate both identities.
            let count = unsafe { pg_sys::list_length(stmt.cmds) };
            for offset in 0..count {
                let command =
                    unsafe { pg_sys::list_nth(stmt.cmds, offset).cast::<pg_sys::AlterTableCmd>() };
                let Some(command) = (unsafe { command.as_ref() }) else {
                    continue;
                };
                if command.subtype != pg_sys::AlterTableType::AT_AttachPartition
                    || command.def.is_null()
                {
                    continue;
                }
                let partition = unsafe { &*command.def.cast::<pg_sys::PartitionCmd>() };
                let attached_oid = unsafe { rangevar_oid(partition.name) };
                reject_if_gated(attached_oid, SOURCE_GATE, "ATTACH PARTITION");
            }
        }
        pg_sys::NodeTag::T_RenameStmt => {
            let stmt = unsafe { &*node.cast::<pg_sys::RenameStmt>() };
            if matches!(
                stmt.renameType,
                pg_sys::ObjectType::OBJECT_TABLE | pg_sys::ObjectType::OBJECT_INDEX
            ) {
                let oid = unsafe { rangevar_oid(stmt.relation) };
                reject_if_gated(oid, SOURCE_GATE | CONTROL_GATE, "RENAME");
            }
        }
        pg_sys::NodeTag::T_AlterObjectSchemaStmt => {
            let stmt = unsafe { &*node.cast::<pg_sys::AlterObjectSchemaStmt>() };
            if matches!(
                stmt.objectType,
                pg_sys::ObjectType::OBJECT_TABLE | pg_sys::ObjectType::OBJECT_INDEX
            ) {
                let oid = unsafe { rangevar_oid(stmt.relation) };
                reject_if_gated(oid, SOURCE_GATE | CONTROL_GATE, "SET SCHEMA");
            }
        }
        pg_sys::NodeTag::T_AlterTableMoveAllStmt => {
            lock_global_gate_serialization(true).unwrap_or_else(|error| pgrx::error!("{error}"));
            reject_if_any_source_gate("ALTER ALL IN TABLESPACE");
        }
        pg_sys::NodeTag::T_CopyStmt => {
            let stmt = unsafe { &*node.cast::<pg_sys::CopyStmt>() };
            if stmt.is_from {
                let oid = unsafe { rangevar_oid(stmt.relation) };
                reject_if_gated(oid, SOURCE_GATE, "COPY FROM");
            }
        }
        pg_sys::NodeTag::T_DropStmt => unsafe { gate_drop(node.cast()) },
        pg_sys::NodeTag::T_TruncateStmt => {
            let stmt = unsafe { &*node.cast::<pg_sys::TruncateStmt>() };
            if stmt.behavior == pg_sys::DropBehavior::DROP_CASCADE {
                // The set reached through foreign keys is discovered only by
                // ExecuteTruncate. Serialize against begin-build, then reject
                // conservatively when any source gate is active.
                lock_global_gate_serialization(true)
                    .unwrap_or_else(|error| pgrx::error!("{error}"));
                reject_if_any_source_gate("TRUNCATE CASCADE");
            }
            let count = unsafe { pg_sys::list_length(stmt.relations) };
            for offset in 0..count {
                let relation =
                    unsafe { pg_sys::list_nth(stmt.relations, offset).cast::<pg_sys::RangeVar>() };
                let oid = unsafe { rangevar_oid(relation) };
                reject_if_gated(oid, SOURCE_GATE, "TRUNCATE");
            }
        }
        pg_sys::NodeTag::T_ClusterStmt => {
            let stmt = unsafe { &*node.cast::<pg_sys::ClusterStmt>() };
            if stmt.relation.is_null() {
                lock_global_gate_serialization(true)
                    .unwrap_or_else(|error| pgrx::error!("{error}"));
                reject_if_any_source_gate("CLUSTER");
            } else {
                let oid = unsafe { rangevar_oid(stmt.relation) };
                reject_if_gated(oid, SOURCE_GATE, "CLUSTER");
            }
        }
        pg_sys::NodeTag::T_VacuumStmt => {
            let stmt = unsafe { &*node.cast::<pg_sys::VacuumStmt>() };
            if unsafe { vacuum_is_full(stmt) } {
                let count = unsafe { pg_sys::list_length(stmt.rels) };
                let mut all_relations = count == 0;
                for offset in 0..count {
                    let relation = unsafe {
                        pg_sys::list_nth(stmt.rels, offset).cast::<pg_sys::VacuumRelation>()
                    };
                    let Some(relation) = (unsafe { relation.as_ref() }) else {
                        continue;
                    };
                    let oid = if relation.oid != pg_sys::InvalidOid {
                        relation.oid
                    } else {
                        unsafe { rangevar_oid(relation.relation) }
                    };
                    if oid == pg_sys::InvalidOid {
                        // PostgreSQL represents unqualified VACUUM with a
                        // placeholder VacuumRelation on some paths rather
                        // than an empty list.
                        all_relations = true;
                    } else {
                        reject_if_gated(oid, SOURCE_GATE, "VACUUM FULL");
                    }
                }
                if all_relations {
                    lock_global_gate_serialization(true)
                        .unwrap_or_else(|error| pgrx::error!("{error}"));
                    reject_if_any_source_gate("VACUUM FULL");
                }
            }
        }
        pg_sys::NodeTag::T_ReindexStmt => {
            let stmt = unsafe { &*node.cast::<pg_sys::ReindexStmt>() };
            if matches!(
                stmt.kind,
                pg_sys::ReindexObjectType::REINDEX_OBJECT_INDEX
                    | pg_sys::ReindexObjectType::REINDEX_OBJECT_TABLE
            ) {
                let oid = unsafe { rangevar_oid(stmt.relation) };
                let required_mask = if stmt.kind == pg_sys::ReindexObjectType::REINDEX_OBJECT_TABLE
                {
                    SOURCE_GATE | CONTROL_GATE
                } else {
                    CONTROL_GATE
                };
                reject_if_gated(oid, required_mask, "REINDEX");
            } else {
                lock_global_gate_serialization(true)
                    .unwrap_or_else(|error| pgrx::error!("{error}"));
                reject_if_any_source_gate("REINDEX");
            }
        }
        pg_sys::NodeTag::T_DropOwnedStmt => {
            lock_global_gate_serialization(true).unwrap_or_else(|error| pgrx::error!("{error}"));
            reject_if_any_source_gate("DROP OWNED");
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn call_next_process_utility(
    pstmt: *mut pg_sys::PlannedStmt,
    query_string: *const core::ffi::c_char,
    read_only_tree: bool,
    context: pg_sys::ProcessUtilityContext::Type,
    params: pg_sys::ParamListInfo,
    query_env: *mut pg_sys::QueryEnvironment,
    dest: *mut pg_sys::DestReceiver,
    qc: *mut pg_sys::QueryCompletion,
) {
    unsafe {
        if let Some(previous) = PREVIOUS_PROCESS_UTILITY_HOOK {
            previous(
                pstmt,
                query_string,
                read_only_tree,
                context,
                params,
                query_env,
                dest,
                qc,
            );
        } else {
            pg_sys::standard_ProcessUtility(
                pstmt,
                query_string,
                read_only_tree,
                context,
                params,
                query_env,
                dest,
                qc,
            );
        }
    }
}

#[pg_guard]
#[allow(clippy::too_many_arguments)]
unsafe extern "C-unwind" fn ec_distann_build_gate_process_utility_hook(
    pstmt: *mut pg_sys::PlannedStmt,
    query_string: *const core::ffi::c_char,
    read_only_tree: bool,
    context: pg_sys::ProcessUtilityContext::Type,
    params: pg_sys::ParamListInfo,
    query_env: *mut pg_sys::QueryEnvironment,
    dest: *mut pg_sys::DestReceiver,
    qc: *mut pg_sys::QueryCompletion,
) {
    if !unsafe { pg_sys::creating_extension } && !INSIDE_GATE_LOOKUP.with(Cell::get) {
        let previous_lock_depth = GLOBAL_UTILITY_LOCK_DEPTH.with(Cell::get);
        return pg_sys::PgTryBuilder::new(AssertUnwindSafe(|| {
            if let Some(pstmt) = unsafe { pstmt.as_ref() } {
                unsafe { gate_utility_node(pstmt.utilityStmt) };
            }
            unsafe {
                call_next_process_utility(
                    pstmt,
                    query_string,
                    read_only_tree,
                    context,
                    params,
                    query_env,
                    dest,
                    qc,
                )
            }
        }))
        .finally(|| release_global_utility_lock_if_acquired(previous_lock_depth))
        .execute();
    }
    unsafe {
        call_next_process_utility(
            pstmt,
            query_string,
            read_only_tree,
            context,
            params,
            query_env,
            dest,
            qc,
        )
    }
}

/// Install both hooks after the SPIRE hook so DistANN can reject unsafe writes
/// before delegating while still preserving every previously installed hook.
pub(crate) unsafe fn register_build_gate_hooks() {
    if unsafe { pg_sys::process_shared_preload_libraries_in_progress } {
        LOADED_DURING_SHARED_PRELOAD.store(true, Ordering::Release);
    }
    if !HOOKS_INSTALLED.swap(true, Ordering::Relaxed) {
        unsafe {
            pg_sys::CacheRegisterSyscacheCallback(
                pg_sys::SysCacheIdentifier::PROCOID as core::ffi::c_int,
                Some(invalidate_gate_function_oid),
                pg_sys::Datum::from(0_usize),
            );
            pg_sys::CacheRegisterRelcacheCallback(
                Some(invalidate_no_active_gate_cache),
                pg_sys::Datum::from(0_usize),
            );
            PREVIOUS_EXECUTOR_START_HOOK = pg_sys::ExecutorStart_hook;
            pg_sys::ExecutorStart_hook = Some(ec_distann_build_gate_executor_start_hook);
            PREVIOUS_PROCESS_UTILITY_HOOK = pg_sys::ProcessUtility_hook;
            pg_sys::ProcessUtility_hook = Some(ec_distann_build_gate_process_utility_hook);
        }
    }
}
