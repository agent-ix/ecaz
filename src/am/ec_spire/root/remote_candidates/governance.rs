fn remote_conninfo_secret_value(conninfo_secret_name: &str) -> Result<String, String> {
    let provider_lookup_key = remote_conninfo_secret_provider_lookup_key(conninfo_secret_name)?;
    match std::env::var(provider_lookup_key) {
        Ok(conninfo) if !conninfo.is_empty() => Ok(conninfo),
        Ok(_) => Err("conninfo_secret_empty".to_owned()),
        Err(_) => Err("conninfo_secret_missing".to_owned()),
    }
}

pub(crate) fn remote_prepared_transaction_registration_warning(
    conninfo_secret_name: &str,
    node_id: i32,
) -> Option<String> {
    let node_id = u32::try_from(node_id).ok()?;
    let conninfo = match remote_conninfo_secret_value(conninfo_secret_name) {
        Ok(conninfo) => conninfo,
        Err(status) => {
            return Some(format!(
                "ec_spire_register_remote_node_descriptor skipped remote \
                 max_prepared_transactions preflight for node_id {node_id}: {status}; \
                 resolve conninfo_secret_name before enabling coordinator-routed writes"
            ));
        }
    };
    let mut client = match remote_search_libpq_connect_with_session_timeouts(
        &conninfo,
        node_id,
        "remote node descriptor max_prepared_transactions preflight",
    ) {
        Ok(client) => client,
        Err(error) => {
            return Some(format!(
                "ec_spire_register_remote_node_descriptor could not check remote \
                 max_prepared_transactions for node_id {node_id}: {error}"
            ));
        }
    };
    let setting = match client.query_one("SHOW max_prepared_transactions", &[]) {
        Ok(row) => row
            .try_get::<_, String>(0)
            .unwrap_or_else(|_| "ec_spire_max_prepared_transactions_decode_failed".to_owned()),
        Err(error) => {
            return Some(format!(
                "ec_spire_register_remote_node_descriptor could not read remote \
                 max_prepared_transactions for node_id {node_id}: {error}"
            ));
        }
    };
    let value = match setting.parse::<i64>() {
        Ok(value) => value,
        Err(_) => {
            return Some(format!(
                "ec_spire_register_remote_node_descriptor could not parse remote \
                 max_prepared_transactions value {setting:?} for node_id {node_id}"
            ));
        }
    };
    if value <= 0 {
        Some(format!(
            "ec_spire_register_remote_node_descriptor remote node_id {node_id} reports \
             max_prepared_transactions = {value}; coordinator-routed SPIRE writes require \
             max_prepared_transactions > 0 and enough free prepared transaction slots"
        ))
    } else {
        None
    }
}

pub(crate) fn remote_search_libpq_connect_with_session_timeouts(
    conninfo: &str,
    node_id: u32,
    context: &str,
) -> Result<postgres::Client, String> {
    let limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
    let mut config = conninfo
        .parse::<postgres::Config>()
        .map_err(|_| format!("ec_spire {context} conninfo parse failed for node_id {node_id}"))?;
    if limits.connect_timeout_ms > 0 {
        config.connect_timeout(std::time::Duration::from_millis(limits.connect_timeout_ms));
    }
    let mut client = config
        .connect(postgres::NoTls)
        .map_err(|_| format!("ec_spire {context} failed to open connection for node_id {node_id}"))?;
    if limits.statement_timeout_ms > 0 {
        let sql = format!("SET statement_timeout = {}", limits.statement_timeout_ms);
        client.batch_execute(&sql).map_err(|_| {
            format!("ec_spire {context} failed to configure statement_timeout for node_id {node_id}")
        })?;
    }
    Ok(client)
}

const SPIRE_REMOTE_SEARCH_LIBPQ_GLOBAL_LOCK_CLASS_BASE: i32 = 730_000_000;
const SPIRE_REMOTE_SEARCH_LIBPQ_NODE_LOCK_CLASS_BASE: i32 = 731_000_000;
#[cfg(any(test, feature = "pg_test"))]
const SPIRE_REMOTE_SEARCH_LIBPQ_GOVERNANCE_TEST_NAMESPACE_STRIDE: i32 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpireRemoteSearchLibpqGovernanceLockKey {
    class_id: i32,
    object_id: i32,
}

#[derive(Debug, Default)]
struct SpireRemoteSearchLibpqGovernancePermit {
    locks: Vec<SpireRemoteSearchLibpqGovernanceLockKey>,
}

impl Drop for SpireRemoteSearchLibpqGovernancePermit {
    fn drop(&mut self) {
        for key in self.locks.iter().rev() {
            let _ = remote_search_libpq_advisory_unlock(*key);
        }
    }
}

fn remote_search_libpq_governance_lock_key(
    class_base: i32,
    object_id: i32,
    slot: u64,
) -> Result<SpireRemoteSearchLibpqGovernanceLockKey, String> {
    let slot = i32::try_from(slot)
        .map_err(|_| "ec_spire remote search executor governance slot exceeds i32".to_owned())?;
    let class_base = remote_search_libpq_governance_class_base(class_base)?;
    let class_id = class_base.checked_add(slot).ok_or_else(|| {
        "ec_spire remote search executor governance advisory lock class overflow".to_owned()
    })?;
    Ok(SpireRemoteSearchLibpqGovernanceLockKey {
        class_id,
        object_id,
    })
}

fn remote_search_libpq_governance_class_base(class_base: i32) -> Result<i32, String> {
    let namespace = options::current_session_remote_search_governance_test_namespace();
    if namespace == 0 {
        return Ok(class_base);
    }

    #[cfg(any(test, feature = "pg_test"))]
    {
        let offset = namespace
            .checked_mul(SPIRE_REMOTE_SEARCH_LIBPQ_GOVERNANCE_TEST_NAMESPACE_STRIDE)
            .ok_or_else(|| {
                "ec_spire remote search executor governance test namespace overflow".to_owned()
            })?;
        return class_base.checked_add(offset).ok_or_else(|| {
            "ec_spire remote search executor governance test class overflow".to_owned()
        });
    }

    #[cfg(not(any(test, feature = "pg_test")))]
    {
        Err("ec_spire remote search executor governance test namespace is unavailable".to_owned())
    }
}

fn remote_search_libpq_advisory_lock_result(
    function_name: &str,
    key: SpireRemoteSearchLibpqGovernanceLockKey,
) -> Result<bool, String> {
    Spi::get_one::<bool>(&format!(
        "SELECT {function_name}({}, {})",
        key.class_id, key.object_id
    ))
    .map_err(|e| {
        format!("ec_spire remote search executor governance advisory lock query failed: {e}")
    })?
    .ok_or_else(|| {
        "ec_spire remote search executor governance advisory lock returned null".to_owned()
    })
}

fn remote_search_libpq_try_advisory_lock(
    key: SpireRemoteSearchLibpqGovernanceLockKey,
) -> Result<bool, String> {
    remote_search_libpq_advisory_lock_result("pg_try_advisory_lock", key)
}

fn remote_search_libpq_advisory_unlock(
    key: SpireRemoteSearchLibpqGovernanceLockKey,
) -> Result<bool, String> {
    remote_search_libpq_advisory_lock_result("pg_advisory_unlock", key)
}

fn remote_search_libpq_try_governance_slot(
    class_base: i32,
    object_id: i32,
    slot_count: u64,
) -> Result<Option<SpireRemoteSearchLibpqGovernanceLockKey>, String> {
    for slot in 0..slot_count {
        let key = remote_search_libpq_governance_lock_key(class_base, object_id, slot)?;
        if remote_search_libpq_try_advisory_lock(key)? {
            return Ok(Some(key));
        }
    }
    Ok(None)
}

fn remote_search_libpq_executor_governance_permit(
    row: &SpireRemoteSearchLibpqDispatchPlanRow,
) -> Result<SpireRemoteSearchLibpqGovernancePermit, String> {
    remote_search_libpq_executor_governance_permit_for_node(row.node_id)
}

fn remote_search_libpq_executor_governance_permit_for_node(
    node_id: u32,
) -> Result<SpireRemoteSearchLibpqGovernancePermit, String> {
    let limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
    let mut permit = SpireRemoteSearchLibpqGovernancePermit::default();

    if limits.has_concurrent_dispatch_cap() {
        let key = remote_search_libpq_try_governance_slot(
            SPIRE_REMOTE_SEARCH_LIBPQ_GLOBAL_LOCK_CLASS_BASE,
            0,
            limits.max_concurrent_dispatches,
        )?
        .ok_or_else(|| {
            format!(
                "ec_spire remote search executor remote_executor_overload global concurrency cap {} is saturated",
                limits.max_concurrent_dispatches
            )
        })?;
        permit.locks.push(key);
    }

    if limits.has_concurrent_dispatch_per_node_cap() {
        let key = remote_search_libpq_try_governance_slot(
            SPIRE_REMOTE_SEARCH_LIBPQ_NODE_LOCK_CLASS_BASE,
            i32::from_ne_bytes(node_id.to_ne_bytes()),
            limits.max_concurrent_dispatches_per_node,
        )?
        .ok_or_else(|| {
            format!(
                "ec_spire remote search executor remote_executor_overload per-node concurrency cap {} is saturated for node_id {}",
                limits.max_concurrent_dispatches_per_node, node_id
            )
        })?;
        permit.locks.push(key);
    }

    Ok(permit)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn remote_search_libpq_global_governance_advisory_key_for_test(
    slot: u64,
) -> (i32, i32) {
    let key = remote_search_libpq_governance_lock_key(
        SPIRE_REMOTE_SEARCH_LIBPQ_GLOBAL_LOCK_CLASS_BASE,
        0,
        slot,
    )
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    (key.class_id, key.object_id)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn remote_search_libpq_node_governance_advisory_key_for_test(
    node_id: u32,
    slot: u64,
) -> (i32, i32) {
    let key = remote_search_libpq_governance_lock_key(
        SPIRE_REMOTE_SEARCH_LIBPQ_NODE_LOCK_CLASS_BASE,
        i32::from_ne_bytes(node_id.to_ne_bytes()),
        slot,
    )
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    (key.class_id, key.object_id)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SpireRemoteEndpointIdentityCacheKey {
    coordinator_index_oid: u32,
    node_id: u32,
    remote_index_regclass: String,
    remote_index_oid: u32,
    descriptor_generation: u64,
    remote_index_identity: Vec<u8>,
    served_epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireRemoteEndpointIdentityCacheEntry {
    protocol_version: String,
    extension_version: String,
    opclass_identity: String,
    storage_format: String,
    assignment_payload_format: String,
    quantizer_profile: String,
    scoring_profile: String,
    profile_fingerprint: String,
}

impl From<SpireRemoteValidatedEndpointIdentity> for SpireRemoteEndpointIdentityCacheEntry {
    fn from(identity: SpireRemoteValidatedEndpointIdentity) -> Self {
        Self {
            protocol_version: identity.protocol_version,
            extension_version: identity.extension_version,
            opclass_identity: identity.opclass_identity,
            storage_format: identity.storage_format,
            assignment_payload_format: identity.assignment_payload_format,
            quantizer_profile: identity.quantizer_profile,
            scoring_profile: identity.scoring_profile,
            profile_fingerprint: identity.profile_fingerprint,
        }
    }
}

#[derive(Debug, Default)]
struct SpireRemoteSearchLibpqExecutorState {
    endpoint_identity_cache:
        HashMap<SpireRemoteEndpointIdentityCacheKey, SpireRemoteEndpointIdentityCacheEntry>,
    endpoint_identity_query_count: u64,
    endpoint_identity_cache_hit_count: u64,
    endpoint_identity_cache_miss_count: u64,
}

impl SpireRemoteSearchLibpqExecutorState {
    fn increment_counter(counter: &mut u64, counter_name: &str) -> Result<(), String> {
        *counter = counter.checked_add(1).ok_or_else(|| {
            format!("ec_spire remote search libpq executor {counter_name} overflow")
        })?;
        Ok(())
    }

    fn endpoint_identity_cache_entry_count(&self) -> Result<u64, String> {
        u64::try_from(self.endpoint_identity_cache.len()).map_err(|_| {
            "ec_spire remote search libpq executor endpoint identity cache size exceeds u64"
                .to_owned()
        })
    }

    fn endpoint_identity_query_count(&self) -> u64 {
        self.endpoint_identity_query_count
    }

    fn endpoint_identity_cache_hit_count(&self) -> u64 {
        self.endpoint_identity_cache_hit_count
    }

    fn endpoint_identity_cache_miss_count(&self) -> u64 {
        self.endpoint_identity_cache_miss_count
    }

    fn lookup_endpoint_identity(
        &mut self,
        key: &SpireRemoteEndpointIdentityCacheKey,
    ) -> Result<bool, String> {
        if self.endpoint_identity_cache.contains_key(key) {
            Self::increment_counter(
                &mut self.endpoint_identity_cache_hit_count,
                "endpoint identity cache hit count",
            )?;
            Ok(true)
        } else {
            Self::increment_counter(
                &mut self.endpoint_identity_cache_miss_count,
                "endpoint identity cache miss count",
            )?;
            Ok(false)
        }
    }

    fn record_endpoint_identity_query(&mut self) -> Result<(), String> {
        Self::increment_counter(
            &mut self.endpoint_identity_query_count,
            "endpoint identity query count",
        )
    }

    fn insert_endpoint_identity(
        &mut self,
        key: SpireRemoteEndpointIdentityCacheKey,
        identity: SpireRemoteValidatedEndpointIdentity,
    ) {
        self.endpoint_identity_cache.insert(key, identity.into());
    }
}

fn remote_search_endpoint_identity_cache_key(
    coordinator_index_oid: pg_sys::Oid,
    row: &SpireRemoteSearchLibpqDispatchPlanRow,
    remote_index_oid: u32,
) -> SpireRemoteEndpointIdentityCacheKey {
    SpireRemoteEndpointIdentityCacheKey {
        coordinator_index_oid: u32::from(coordinator_index_oid),
        node_id: row.node_id,
        remote_index_regclass: row.remote_index_regclass.clone(),
        remote_index_oid,
        descriptor_generation: row.descriptor_generation,
        remote_index_identity: row.remote_index_identity.clone(),
        served_epoch: row.requested_epoch,
    }
}

fn validate_remote_search_libpq_endpoint_identity_for_dispatch(
    client: &mut postgres::Client,
    coordinator_index_oid: pg_sys::Oid,
    remote_index_oid: u32,
    row: &SpireRemoteSearchLibpqDispatchPlanRow,
    executor_state: &mut SpireRemoteSearchLibpqExecutorState,
) -> Result<(), String> {
    let cache_key = remote_search_endpoint_identity_cache_key(coordinator_index_oid, row, remote_index_oid);
    if executor_state.lookup_endpoint_identity(&cache_key)? {
        return Ok(());
    }

    executor_state.record_endpoint_identity_query()?;
    let endpoint_identity_row = client
        .query_one(
            SPIRE_REMOTE_SEARCH_ENDPOINT_IDENTITY_SQL_TEMPLATE,
            &[&remote_index_oid],
        )
        .map_err(|_| {
            format!(
                "ec_spire remote search libpq executor endpoint identity query failed for node_id {}",
                row.node_id
            )
        })?;
    let endpoint_identity = validate_remote_search_endpoint_identity_row(&endpoint_identity_row)?;
    if endpoint_identity.profile_fingerprint_bytes.as_slice() != row.remote_index_identity.as_slice() {
        return Err(format!(
            "ec_spire remote search executor remote_index_identity does not match endpoint profile_fingerprint for node_id {}",
            row.node_id
        ));
    }
    executor_state.insert_endpoint_identity(cache_key, endpoint_identity);
    Ok(())
}

