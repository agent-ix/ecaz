fn quote_sql_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn coordinator_insert_prepared_gid(
    index_oid: pg_sys::Oid,
    node_id: u32,
    served_epoch: u64,
) -> String {
    // SAFETY: This helper is called while executing inside a PostgreSQL backend
    // transaction; `GetTopTransactionId` only reads/assigns the current top XID.
    let transaction_id = unsafe { pg_sys::GetTopTransactionId() };
    format!(
        "ec_spire_insert_{}_{}_{}_{}",
        u32::from(index_oid),
        node_id,
        served_epoch,
        u32::from(transaction_id)
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpirePreparedGidParts {
    index_oid: u32,
    node_id: u32,
    served_epoch: u64,
    xid: u64,
}

fn parse_spire_prepared_gid(gid: &str) -> Option<SpirePreparedGidParts> {
    let suffix = gid.strip_prefix("ec_spire_insert_")?;
    let mut parts = suffix.split('_');
    let index_oid = parts.next()?.parse::<u32>().ok()?;
    let node_id = parts.next()?.parse::<u32>().ok()?;
    let served_epoch = parts.next()?.parse::<u64>().ok()?;
    let xid = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(SpirePreparedGidParts {
        index_oid,
        node_id,
        served_epoch,
        xid,
    })
}

fn coordinator_prepared_xact_intent_state_is_valid(state: &str) -> bool {
    matches!(
        state,
        SPIRE_PREPARED_XACT_INTENT_PREPARE_REQUESTED
            | SPIRE_PREPARED_XACT_INTENT_PREPARE_ACKED
            | SPIRE_PREPARED_XACT_INTENT_COMMIT_LOCAL
            | SPIRE_PREPARED_XACT_INTENT_ROLLBACK_LOCAL
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpirePreparedXactIntentTransitionContext {
    RemotePrepareAck,
    LocalCommitRecorded,
    ReaperRollback,
}

fn coordinator_prepared_xact_intent_transition_is_valid(
    current_state: &str,
    next_state: &str,
    context: SpirePreparedXactIntentTransitionContext,
) -> bool {
    matches!(
        (context, current_state, next_state),
        (
            SpirePreparedXactIntentTransitionContext::RemotePrepareAck,
            SPIRE_PREPARED_XACT_INTENT_PREPARE_REQUESTED,
            SPIRE_PREPARED_XACT_INTENT_PREPARE_ACKED,
        ) | (
            SpirePreparedXactIntentTransitionContext::LocalCommitRecorded,
            SPIRE_PREPARED_XACT_INTENT_PREPARE_ACKED,
            SPIRE_PREPARED_XACT_INTENT_COMMIT_LOCAL,
        ) | (
            SpirePreparedXactIntentTransitionContext::ReaperRollback,
            SPIRE_PREPARED_XACT_INTENT_PREPARE_REQUESTED
                | SPIRE_PREPARED_XACT_INTENT_PREPARE_ACKED,
            SPIRE_PREPARED_XACT_INTENT_ROLLBACK_LOCAL,
        )
    )
}

fn coordinator_prepared_xact_intent_record_prepare_requested(
    index_oid: pg_sys::Oid,
    node_id: u32,
    served_epoch: u64,
    gid: &str,
) -> Result<(), String> {
    let parts = parse_spire_prepared_gid(gid).ok_or_else(|| {
        format!("ec_spire prepared transaction intent cannot parse SPIRE gid {gid}")
    })?;
    let index_oid_u32 = u32::from(index_oid);
    if parts.index_oid != index_oid_u32
        || parts.node_id != node_id
        || parts.served_epoch != served_epoch
    {
        return Err(format!(
            "ec_spire prepared transaction intent gid {gid} does not match dispatch index_oid {index_oid_u32}, node_id {node_id}, served_epoch {served_epoch}"
        ));
    }
    let sql = format!(
        "INSERT INTO ec_spire_remote_prepared_xact_intent \
             (index_oid, node_id, served_epoch, xid, gid, intent_state) \
         VALUES ('{}'::oid, {}::integer, {}::bigint, {}::bigint, {}, {}) \
         ON CONFLICT (gid) DO UPDATE SET \
             index_oid = EXCLUDED.index_oid, \
             node_id = EXCLUDED.node_id, \
             served_epoch = EXCLUDED.served_epoch, \
             xid = EXCLUDED.xid, \
             intent_state = EXCLUDED.intent_state, \
             updated_at = clock_timestamp()",
        index_oid_u32,
        node_id,
        served_epoch,
        parts.xid,
        quote_sql_literal(gid),
        quote_sql_literal(SPIRE_PREPARED_XACT_INTENT_PREPARE_REQUESTED)
    );
    Spi::run(&sql).map_err(|e| {
        format!("ec_spire prepared transaction intent prepare_requested record failed: {e}")
    })
}

fn coordinator_prepared_xact_intent_mark(
    gid: &str,
    state: &str,
    context: SpirePreparedXactIntentTransitionContext,
) -> Result<(), String> {
    if !coordinator_prepared_xact_intent_state_is_valid(state) {
        return Err(format!(
            "ec_spire prepared transaction intent state {state} is not supported"
        ));
    }
    let _ = context;
    #[cfg(test)]
    if let Some(current_state) = coordinator_prepared_xact_intent_state(gid)? {
        if !coordinator_prepared_xact_intent_transition_is_valid(&current_state, state, context) {
            return Err(format!(
                "ec_spire prepared transaction intent transition {current_state} -> {state} is invalid for {context:?}"
            ));
        }
    }
    let sql = format!(
        "UPDATE ec_spire_remote_prepared_xact_intent \
            SET intent_state = {}, updated_at = clock_timestamp() \
          WHERE gid = {}",
        quote_sql_literal(state),
        quote_sql_literal(gid)
    );
    Spi::run(&sql)
        .map_err(|e| format!("ec_spire prepared transaction intent state update failed: {e}"))
}

fn coordinator_xid_is_live(xid: u64) -> Result<bool, String> {
    let sql = format!(
        "SELECT EXISTS ( \
             SELECT 1 \
               FROM pg_stat_activity \
              WHERE backend_xid::text = {} \
                 OR backend_xmin::text = {} \
         )",
        quote_sql_literal(&xid.to_string()),
        quote_sql_literal(&xid.to_string())
    );
    Spi::get_one::<bool>(&sql)
        .map_err(|e| format!("ec_spire prepared transaction xid liveness check failed: {e}"))?
        .ok_or_else(|| "ec_spire prepared transaction xid liveness check returned no row".to_owned())
}

fn coordinator_prepared_xact_intent_state(gid: &str) -> Result<Option<String>, String> {
    let sql = format!(
        "SELECT intent_state \
           FROM ec_spire_remote_prepared_xact_intent \
          WHERE gid = {}",
        quote_sql_literal(gid)
    );
    Spi::get_one::<String>(&sql)
        .map_err(|e| format!("ec_spire prepared transaction intent lookup failed: {e}"))
}

fn coordinator_prepared_xact_reaper_conninfo(node_id: u32) -> Result<String, String> {
    let sql = format!(
        "SELECT conninfo_secret_name \
           FROM ec_spire_remote_node_descriptor \
          WHERE node_id = {}::integer \
            AND descriptor_state = 'active' \
          ORDER BY descriptor_generation DESC \
          LIMIT 1",
        node_id
    );
    let conninfo_secret_name = Spi::get_one::<String>(&sql)
        .map_err(|e| {
            format!("ec_spire prepared transaction reaper descriptor lookup failed: {e}")
        })?
        .ok_or_else(|| {
            format!(
                "ec_spire prepared transaction reaper found no active descriptor for node_id {node_id}"
            )
        })?;
    remote_conninfo_secret_value(&conninfo_secret_name).map_err(|status| {
        format!(
            "ec_spire prepared transaction reaper conninfo secret for node_id {node_id} is not resolved: {status}"
        )
    })
}

fn coordinator_prepared_xact_reaper_active_node_ids() -> Result<Vec<u32>, String> {
    let sql = "SELECT coalesce(string_agg(node_id::text, ',' ORDER BY node_id), '') \
                 FROM ( \
                       SELECT DISTINCT node_id \
                         FROM ec_spire_remote_node_descriptor \
                        WHERE descriptor_state = 'active' \
                      ) nodes";
    let csv = Spi::get_one::<String>(sql)
        .map_err(|e| {
            format!("ec_spire prepared transaction reaper active node lookup failed: {e}")
        })?
        .unwrap_or_default();
    if csv.is_empty() {
        return Ok(Vec::new());
    }
    csv.split(',')
        .map(|part| {
            part.parse::<u32>().map_err(|e| {
                format!(
                    "ec_spire prepared transaction reaper active node_id decode failed for {part}: {e}"
                )
            })
        })
        .collect()
}

pub(crate) fn reap_orphaned_remote_prepared_xacts(
    node_id: u32,
) -> Result<Vec<SpireRemotePreparedXactReaperRow>, String> {
    let conninfo = coordinator_prepared_xact_reaper_conninfo(node_id)?;
    let mut client = remote_search_libpq_connect_with_session_timeouts(
        &conninfo,
        node_id,
        "prepared transaction reaper",
    )?;
    let prepared_rows = client
        .query(
            "SELECT gid \
               FROM pg_prepared_xacts \
              WHERE gid LIKE 'ec_spire_insert_%' \
              ORDER BY prepared, gid",
            &[],
        )
        .map_err(|e| {
            format!("ec_spire prepared transaction reaper pg_prepared_xacts scan failed: {e}")
        })?;
    let mut rows = Vec::with_capacity(prepared_rows.len());
    for prepared_row in prepared_rows {
        let gid = prepared_row
            .try_get::<_, String>("gid")
            .map_err(|e| format!("ec_spire prepared transaction reaper gid decode failed: {e}"))?;
        let Some(parts) = parse_spire_prepared_gid(&gid) else {
            rows.push(SpireRemotePreparedXactReaperRow {
                node_id,
                index_oid: 0,
                served_epoch: 0,
                xid: 0,
                gid,
                intent_state: "unparseable".to_owned(),
                coordinator_xid_live: false,
                action: SPIRE_PREPARED_XACT_REAPER_SKIPPED_UNPARSEABLE_GID.to_owned(),
                detail: "gid does not match ec_spire_insert_<index_oid>_<node_id>_<served_epoch>_<top_xid>".to_owned(),
            });
            continue;
        };
        if parts.node_id != node_id {
            rows.push(SpireRemotePreparedXactReaperRow {
                node_id,
                index_oid: parts.index_oid,
                served_epoch: parts.served_epoch,
                xid: parts.xid,
                gid,
                intent_state: "node_mismatch".to_owned(),
                coordinator_xid_live: false,
                action: SPIRE_PREPARED_XACT_REAPER_SKIPPED_NODE_MISMATCH.to_owned(),
                detail: format!(
                    "remote gid node_id {} does not match requested node_id {node_id}",
                    parts.node_id
                ),
            });
            continue;
        }
        let intent_state = coordinator_prepared_xact_intent_state(&gid)?
            .unwrap_or_else(|| "missing_intent".to_owned());
        let coordinator_xid_live = coordinator_xid_is_live(parts.xid)?;
        let (action, detail) = if intent_state == SPIRE_PREPARED_XACT_INTENT_COMMIT_LOCAL {
            (
                SPIRE_PREPARED_XACT_REAPER_SKIPPED_COMMIT_LOCAL.to_owned(),
                "coordinator recorded commit_local; operator must resolve any failed remote commit manually".to_owned(),
            )
        } else if coordinator_xid_live {
            (
                SPIRE_PREPARED_XACT_REAPER_SKIPPED_XID_LIVE.to_owned(),
                "coordinator top transaction is still visible in pg_stat_activity".to_owned(),
            )
        } else {
            match client.batch_execute(&format!(
                "ROLLBACK PREPARED {}",
                quote_sql_literal(&gid)
            )) {
                Ok(()) => {
                    let _ = coordinator_prepared_xact_intent_mark(
                        &gid,
                        SPIRE_PREPARED_XACT_INTENT_ROLLBACK_LOCAL,
                        SpirePreparedXactIntentTransitionContext::ReaperRollback,
                    );
                    if intent_state == "missing_intent" {
                        (
                            SPIRE_PREPARED_XACT_REAPER_ROLLED_BACK_MISSING_INTENT.to_owned(),
                            "rolled back parsed SPIRE gid with no coordinator intent row because coordinator top transaction is no longer live".to_owned(),
                        )
                    } else {
                        (
                            SPIRE_PREPARED_XACT_REAPER_ROLLED_BACK.to_owned(),
                            "rolled back remote prepared transaction".to_owned(),
                        )
                    }
                }
                Err(error) => (
                    SPIRE_PREPARED_XACT_REAPER_ROLLBACK_FAILED.to_owned(),
                    format!("ROLLBACK PREPARED failed: {error}"),
                ),
            }
        };
        rows.push(SpireRemotePreparedXactReaperRow {
            node_id,
            index_oid: parts.index_oid,
            served_epoch: parts.served_epoch,
            xid: parts.xid,
            gid,
            intent_state,
            coordinator_xid_live,
            action,
            detail,
        });
    }
    Ok(rows)
}

pub(crate) fn reap_orphaned_remote_prepared_xacts_all(
) -> Result<Vec<SpireRemotePreparedXactReaperRow>, String> {
    let mut rows = Vec::new();
    for node_id in coordinator_prepared_xact_reaper_active_node_ids()? {
        rows.extend(reap_orphaned_remote_prepared_xacts(node_id)?);
    }
    Ok(rows)
}

fn coordinator_insert_resolve_remote_prepared(
    conninfo: String,
    node_id: u32,
    gid: String,
    commit: bool,
) {
    let context = if commit {
        "coordinator insert remote prepared commit callback"
    } else {
        "coordinator insert remote prepared rollback callback"
    };
    let Ok(mut client) =
        remote_search_libpq_connect_with_session_timeouts(&conninfo, node_id, context)
    else {
        return;
    };
    let command = if commit {
        "COMMIT PREPARED"
    } else {
        "ROLLBACK PREPARED"
    };
    let _ = client.batch_execute(&format!("{command} {}", quote_sql_literal(&gid)));
}
