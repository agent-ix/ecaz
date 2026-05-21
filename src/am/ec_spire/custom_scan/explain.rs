#[derive(Debug, Clone, Copy)]
struct SpireCustomScanExplainContext {
    remote_fanout: u64,
    nprobe: u64,
    rerank_width: i64,
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_explain_custom_scan(
    node: *mut pg_sys::CustomScanState,
    _ancestors: *mut pg_sys::List,
    es: *mut pg_sys::ExplainState,
) {
    if node.is_null() || es.is_null() {
        return;
    }

    // SAFETY: PostgreSQL invokes ExplainCustomScan with a live provider
    // CustomScanState for the duration of this callback.
    let plan = unsafe { CustomScanPlan::from_state(node) };
    let index_oid = plan.index_oid();
    let context = custom_scan_explain_context(index_oid);

    // SAFETY: `es` is the non-null ExplainState supplied by PostgreSQL for the
    // duration of this callback; property names and values are static C strings.
    unsafe {
        pg_sys::ExplainPropertyText(c"node".as_ptr(), c"EcSpireDistributedScan".as_ptr(), es);
        pg_sys::ExplainPropertyUInteger(
            c"remote_fanout".as_ptr(),
            std::ptr::null(),
            context.remote_fanout,
            es,
        );
        // Minimal Phase 12b contract: this is a stable shape marker, not a
        // live transport probe.
        pg_sys::ExplainPropertyText(c"tuple_transport_status".as_ptr(), c"ready".as_ptr(), es);
        pg_sys::ExplainPropertyUInteger(c"nprobe".as_ptr(), std::ptr::null(), context.nprobe, es);
        pg_sys::ExplainPropertyInteger(
            c"rerank_width".as_ptr(),
            std::ptr::null(),
            context.rerank_width,
            es,
        );
    }
}

fn custom_scan_explain_context(index_oid: pg_sys::Oid) -> SpireCustomScanExplainContext {
    if index_oid == pg_sys::InvalidOid {
        return SpireCustomScanExplainContext {
            remote_fanout: 0,
            nprobe: 0,
            rerank_width: 0,
        };
    }

    let Some(index_relation) =
        crate::storage::relation_guard::IndexRelationGuard::try_access_share(index_oid)
    else {
        return SpireCustomScanExplainContext {
            remote_fanout: 0,
            nprobe: 0,
            rerank_width: 0,
        };
    };

    // SAFETY: The relation pointer is owned by `IndexRelationGuard` and
    // remains open under AccessShareLock while these helpers read relation
    // metadata.
    let index = unsafe { super::live_index_relation(index_relation.as_ptr()) };
    let eligibility =
        custom_scan_index_eligibility_result(index).unwrap_or_else(|e| pgrx::error!("{e}"));
    let relation_options = super::options::relation_options(index_relation.as_ptr());
    let configured_nlists = u32::try_from(relation_options.nlists).unwrap_or(0);
    let relation_nprobe = u32::try_from(relation_options.nprobe).unwrap_or(0);
    let nprobe = super::options::resolve_scan_nprobe(configured_nlists, relation_nprobe);
    let rerank_width = super::options::resolve_scan_rerank_width(relation_options.rerank_width);

    SpireCustomScanExplainContext {
        remote_fanout: eligibility.remote_available_node_count,
        nprobe: u64::from(nprobe.effective_nprobe),
        rerank_width: i64::from(rerank_width.effective_rerank_width),
    }
}
