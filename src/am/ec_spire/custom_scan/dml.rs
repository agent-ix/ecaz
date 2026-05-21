fn custom_scan_validate_dml_column_metadata(
    mode: SpireCustomScanPlanMode,
    updated_columns: &[String],
    projected_columns: &[String],
) -> Result<(), String> {
    match mode {
        SpireCustomScanPlanMode::DmlPkSelectTuplePayload => {
            if projected_columns.is_empty() {
                return Err(
                    "EcSpireDistributedScan DML PK SELECT plan requires projected columns"
                        .to_owned(),
                );
            }
            if !updated_columns.is_empty() {
                return Err(
                    "EcSpireDistributedScan DML PK SELECT plan must not update columns".to_owned(),
                );
            }
        }
        SpireCustomScanPlanMode::DmlUpdateTuplePayload => {
            if updated_columns.is_empty() {
                return Err(
                    "EcSpireDistributedScan DML UPDATE plan requires updated columns".to_owned(),
                );
            }
            if !projected_columns.is_empty() {
                return Err(
                    "EcSpireDistributedScan DML UPDATE plan must not project columns".to_owned(),
                );
            }
        }
        SpireCustomScanPlanMode::DmlDeleteTuplePayload => {
            if !updated_columns.is_empty() || !projected_columns.is_empty() {
                return Err(
                    "EcSpireDistributedScan DML DELETE plan must not carry column payload metadata"
                        .to_owned(),
                );
            }
        }
        SpireCustomScanPlanMode::VectorOrderLimit => {}
    }
    Ok(())
}

fn custom_scan_dml_frontdoor_mode_for_plan_mode(
    mode: SpireCustomScanPlanMode,
) -> Result<super::SpireDmlFrontdoorCustomScanMode, String> {
    match mode {
        SpireCustomScanPlanMode::DmlPkSelectTuplePayload => {
            Ok(super::SpireDmlFrontdoorCustomScanMode::CoordinatorPkSelectTuplePayload)
        }
        SpireCustomScanPlanMode::DmlUpdateTuplePayload => {
            Ok(super::SpireDmlFrontdoorCustomScanMode::CoordinatorUpdateTuplePayload)
        }
        SpireCustomScanPlanMode::DmlDeleteTuplePayload => {
            Ok(super::SpireDmlFrontdoorCustomScanMode::CoordinatorDeleteTuplePayload)
        }
        SpireCustomScanPlanMode::VectorOrderLimit => {
            Err("EcSpireDistributedScan vector plan mode has no DML primitive".to_owned())
        }
    }
}

fn custom_scan_dml_primitive_name(mode: SpireCustomScanPlanMode) -> Result<&'static str, String> {
    match mode {
        SpireCustomScanPlanMode::DmlPkSelectTuplePayload => {
            Ok("ec_spire_forward_coordinator_select_tuple_payload")
        }
        SpireCustomScanPlanMode::DmlUpdateTuplePayload => {
            Ok("ec_spire_forward_coordinator_update_tuple_payload")
        }
        SpireCustomScanPlanMode::DmlDeleteTuplePayload => {
            Ok("ec_spire_prepare_coordinator_delete_tuple_payload")
        }
        SpireCustomScanPlanMode::VectorOrderLimit => {
            Err("EcSpireDistributedScan vector plan mode has no DML primitive".to_owned())
        }
    }
}

fn custom_scan_dml_primitive_invocation_from_parts(
    index_oid: pg_sys::Oid,
    mode: SpireCustomScanPlanMode,
    pk_column: &str,
    pk_value: [u8; 8],
    updated_columns: &[String],
    projected_columns: &[String],
) -> Result<super::dml_frontdoor::SpireDmlFrontdoorPrimitiveInvocation, String> {
    if index_oid == pg_sys::InvalidOid {
        return Err(
            "EcSpireDistributedScan DML primitive invocation requires index OID".to_owned(),
        );
    }
    if pk_column.is_empty() {
        return Err(
            "EcSpireDistributedScan DML primitive invocation requires PK column".to_owned(),
        );
    }
    custom_scan_validate_dml_column_metadata(mode, updated_columns, projected_columns)?;
    Ok(super::dml_frontdoor::SpireDmlFrontdoorPrimitiveInvocation {
        index_oid,
        mode: custom_scan_dml_frontdoor_mode_for_plan_mode(mode)?,
        primitive: custom_scan_dml_primitive_name(mode)?,
        pk_column: pk_column.to_owned(),
        pk_value,
        updated_columns: updated_columns.to_vec(),
        projected_columns: projected_columns.to_vec(),
    })
}

fn custom_scan_dml_primitive_invocation(
    state: &SpireCustomScanExecState,
) -> Result<super::dml_frontdoor::SpireDmlFrontdoorPrimitiveInvocation, String> {
    custom_scan_dml_primitive_invocation_from_parts(
        state.index_oid,
        state.mode,
        &state.dml_pk_column,
        state.dml_pk_value,
        &state.dml_updated_columns,
        &state.dml_projected_columns,
    )
}

fn custom_scan_exec_state_mut<'a>(
    state: *mut SpireCustomScanExecState,
    context: &str,
) -> &'a mut SpireCustomScanExecState {
    // SAFETY: executor callbacks pass the provider-owned CustomScan exec state
    // for the duration of the current callback.
    unsafe { state.as_mut() }
        .unwrap_or_else(|| pgrx::error!("EcSpireDistributedScan {context} state is NULL"))
}

fn custom_scan_top_k_from_plan(plan: CustomScanPlan<'_>) -> usize {
    let custom_private = plan.custom_private("private LIMIT");
    let raw = custom_private
        .nth_oid(2)
        .unwrap_or_else(|| pgrx::error!("EcSpireDistributedScan plan is missing private LIMIT"));
    usize::try_from(raw.to_u32())
        .unwrap_or_else(|_| pgrx::error!("EcSpireDistributedScan plan LIMIT is out of range"))
}

unsafe fn custom_scan_query_from_plan(
    node: *mut pg_sys::CustomScanState,
    custom_scan: *mut pg_sys::CustomScan,
) -> Vec<f32> {
    // SAFETY: executor passes the live CustomScanState; the expression was
    // extracted from the provider-owned CustomScan custom_exprs list.
    unsafe {
        let expr = CustomScanExprList::from_custom_scan(custom_scan, "ORDER BY query expression")
            .expr(0, "ORDER BY query expression");
        let Some(query_expr) = CustomScanExpr::new(expr) else {
            pgrx::error!("EcSpireDistributedScan plan has a null ORDER BY query expression");
        };
        let datum = match query_expr.tag() {
            pg_sys::NodeTag::T_Const => {
                return query_expr.query_values_from_const().unwrap_or_else(|| {
                    pgrx::error!(
                        "EcSpireDistributedScan requires a non-null finite real[] ORDER BY query"
                    )
                });
            }
            pg_sys::NodeTag::T_Param => {
                let expr_state = pg_sys::ExecInitExpr(expr, &mut (*node).ss.ps);
                if expr_state.is_null() {
                    pgrx::error!("EcSpireDistributedScan failed to initialize ORDER BY query parameter expression");
                }
                let eval = (*expr_state).evalfunc.unwrap_or_else(|| {
                    pgrx::error!(
                        "EcSpireDistributedScan ORDER BY query expression has no evaluator"
                    )
                });
                let mut is_null = false;
                let datum = eval(expr_state, (*node).ss.ps.ps_ExprContext, &mut is_null);
                if is_null {
                    pgrx::error!(
                        "EcSpireDistributedScan ORDER BY query parameter must not be NULL"
                    );
                }
                datum
            }
            _ => pgrx::error!(
                "EcSpireDistributedScan requires a constant or parameter real[] ORDER BY query"
            ),
        };
        let values = Vec::<f32>::from_polymorphic_datum(datum, false, pg_sys::FLOAT4ARRAYOID)
            .filter(|values| !values.is_empty() && values.iter().all(|value| value.is_finite()))
            .unwrap_or_else(|| {
                pgrx::error!(
                    "EcSpireDistributedScan requires a finite real[] ORDER BY query parameter"
                )
            });
        values
    }
}

unsafe fn custom_scan_tuple_payload_columns(
    node: *mut pg_sys::CustomScanState,
    custom_scan: *mut pg_sys::CustomScan,
) -> Vec<String> {
    // SAFETY: executor passes live CustomScanState/CustomScan pointers; relation
    // and targetlist metadata are read only for tuple payload column discovery.
    unsafe {
        let relation = custom_scan_current_relation(node, "tuple payload columns");
        let tuple_desc = crate::storage::relation::relation_tuple_desc_copy(relation);
        let tuple_desc = TupleDescView::from_raw(tuple_desc.as_ptr(), "EcSpireDistributedScan")
            .unwrap_or_else(|error| pgrx::error!("{error}"));
        let mut attr_numbers = std::collections::BTreeSet::new();
        let mut can_narrow_projection = false;
        if !custom_scan.is_null() && !(*custom_scan).scan.plan.targetlist.is_null() {
            let target_list =
                PgList::<pg_sys::TargetEntry>::from_pg((*custom_scan).scan.plan.targetlist);
            can_narrow_projection = true;
            for target_entry in target_list.iter_ptr() {
                let Some(target_entry) = target_entry.as_ref() else {
                    continue;
                };
                if target_entry.resjunk || target_entry.expr.is_null() {
                    continue;
                }
                let expr = target_entry.expr.cast::<pg_sys::Node>();
                if (*expr).type_ != pg_sys::NodeTag::T_Var {
                    can_narrow_projection = false;
                    break;
                }
                let var = &*target_entry.expr.cast::<pg_sys::Var>();
                if var.varattno > 0 {
                    attr_numbers.insert(var.varattno);
                } else {
                    can_narrow_projection = false;
                    break;
                }
            }
        }
        if !can_narrow_projection {
            attr_numbers.clear();
        }
        let natts = tuple_desc.natts();
        let mut columns = Vec::with_capacity(usize::try_from(natts).unwrap_or(0));
        for attr_index in 0..natts {
            let Some(attr) = tuple_desc
                .attribute(attr_index)
                .unwrap_or_else(|error| pgrx::error!("{error}"))
            else {
                continue;
            };
            if !attr_numbers.is_empty() && !attr_numbers.contains(&attr.attnum) {
                continue;
            }
            custom_scan_validate_tuple_payload_attr(&attr);
            columns.push(attr.name);
        }
        columns
    }
}

#[derive(Clone, Copy)]
struct CustomScanExprList<'a> {
    custom_exprs: *mut pg_sys::List,
    _custom_scan: std::marker::PhantomData<&'a pg_sys::CustomScan>,
}

impl<'a> CustomScanExprList<'a> {
    unsafe fn from_custom_scan(custom_scan: *mut pg_sys::CustomScan, label: &str) -> Self {
        // SAFETY: caller guarantees custom_scan is a live provider-owned
        // CustomScan plan node for the current executor callback.
        let Some(custom_scan) = (unsafe { custom_scan_pg_ref(custom_scan) }) else {
            pgrx::error!("EcSpireDistributedScan plan is missing {label}");
        };
        if custom_scan.custom_exprs.is_null() {
            pgrx::error!("EcSpireDistributedScan plan is missing {label}");
        }
        Self {
            custom_exprs: custom_scan.custom_exprs,
            _custom_scan: std::marker::PhantomData,
        }
    }

    fn len(self) -> i32 {
        // SAFETY: this view is constructed only from a live provider-owned
        // CustomScan expression list.
        unsafe { custom_scan_list_len(self.custom_exprs) }
            .unwrap_or_else(|| pgrx::error!("EcSpireDistributedScan plan expression list is NULL"))
    }

    fn expr(self, offset: i32, label: &str) -> *mut pg_sys::Expr {
        // SAFETY: this view is constructed only from a live provider-owned
        // CustomScan expression list and the helper bounds-checks `offset`.
        let expr = unsafe { custom_scan_list_nth_node(self.custom_exprs, offset) }
            .unwrap_or_else(|| pgrx::error!("EcSpireDistributedScan plan is missing {label}"));
        if expr.is_null() {
            pgrx::error!("EcSpireDistributedScan plan has a null {label}");
        }
        expr.cast::<pg_sys::Expr>()
    }
}

fn custom_scan_validate_tuple_payload_attr(attr: &TupleSlotAttribute) {
    let mut typreceive = pg_sys::InvalidOid;
    let mut typioparam = pg_sys::InvalidOid;
    // SAFETY: attr metadata was copied from a live tuple descriptor attribute;
    // PostgreSQL only reads type-cache metadata for this type OID.
    unsafe { pg_sys::getTypeBinaryInputInfo(attr.typid, &mut typreceive, &mut typioparam) };
    if typreceive == pg_sys::InvalidOid {
        pgrx::error!(
            "EcSpireDistributedScan tuple payload column \"{}\" lacks binary receive support",
            attr.name
        );
    }
}

fn custom_scan_payload_attr_io(
    tuple_desc: &TupleDescView<'_>,
) -> Vec<Option<SpireCustomScanPayloadAttrIo>> {
    let natts = tuple_desc.natts();
    let mut inputs = Vec::with_capacity(usize::try_from(natts).unwrap_or(0));
    for attr_index in 0..natts {
        let Some(attr) = tuple_desc
            .attribute(attr_index)
            .unwrap_or_else(|error| pgrx::error!("{error}"))
        else {
            inputs.push(None);
            continue;
        };
        let mut typinput = pg_sys::InvalidOid;
        let mut typioparam = pg_sys::InvalidOid;
        let mut typreceive = pg_sys::InvalidOid;
        let mut receive_typioparam = pg_sys::InvalidOid;
        // SAFETY: attr metadata was copied from a live tuple descriptor
        // attribute. PostgreSQL only reads type-cache metadata for the type OID,
        // and fmgr_info initializes both FmgrInfo outputs before they are read.
        let attr_io = unsafe {
            pg_sys::getTypeInputInfo(attr.typid, &mut typinput, &mut typioparam);
            let mut input_flinfo =
                std::mem::MaybeUninit::<pg_sys::FmgrInfo>::zeroed().assume_init();
            pg_sys::fmgr_info(typinput, &mut input_flinfo);
            pg_sys::getTypeBinaryInputInfo(attr.typid, &mut typreceive, &mut receive_typioparam);
            let mut receive_flinfo =
                std::mem::MaybeUninit::<pg_sys::FmgrInfo>::zeroed().assume_init();
            pg_sys::fmgr_info(typreceive, &mut receive_flinfo);
            SpireCustomScanPayloadAttrIo {
                input_flinfo,
                input_typioparam: typioparam,
                receive_flinfo,
                receive_typioparam,
                typmod: attr.typmod,
            }
        };
        inputs.push(Some(attr_io));
    }
    inputs
}

unsafe fn custom_scan_dml_pk_column(node: *mut pg_sys::CustomScanState) -> String {
    let relation = unsafe { custom_scan_current_relation(node, "DML path") };
    let relation_oid = crate::storage::relation::relation_oid(relation);
    let context = super::dml_frontdoor_relation_context_catalog_row(relation_oid)
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    context.pk_column.unwrap_or_else(|| {
        pgrx::error!("EcSpireDistributedScan DML path relation has no PK column")
    })
}

unsafe fn custom_scan_dml_pk_value_from_plan(
    node: *mut pg_sys::CustomScanState,
    custom_scan: *mut pg_sys::CustomScan,
) -> [u8; 8] {
    // SAFETY: executor passes a live CustomScanState and expr comes from the
    // provider-owned DML custom_exprs list.
    let value = unsafe {
        let expr = CustomScanExprList::from_custom_scan(custom_scan, "PK expression")
            .expr(0, "PK expression");
        custom_scan_bigint_expr_value(node, expr)
    };
    super::dml_frontdoor_bigint_pk_value_bytes(value)
}

unsafe fn custom_scan_dml_update_value_exprs_from_plan(
    custom_scan: *mut pg_sys::CustomScan,
    expected_count: usize,
) -> Vec<*mut pg_sys::Expr> {
    // SAFETY: caller guarantees custom_scan is a live provider-owned
    // CustomScan plan node carrying the DML UPDATE expression list.
    unsafe {
        let custom_exprs =
            CustomScanExprList::from_custom_scan(custom_scan, "DML UPDATE value expressions");
        let expected_len = i32::try_from(expected_count.saturating_add(1)).unwrap_or_else(|_| {
            pgrx::error!("EcSpireDistributedScan DML UPDATE expression list is too wide")
        });
        if custom_exprs.len() != expected_len {
            pgrx::error!(
                "EcSpireDistributedScan DML UPDATE plan expression count does not match updated columns"
            );
        }
        let mut exprs = Vec::with_capacity(expected_count);
        for offset in 1..expected_len {
            exprs.push(custom_exprs.expr(offset, "DML UPDATE value expression"));
        }
        exprs
    }
}

unsafe fn custom_scan_bigint_expr_value(
    node: *mut pg_sys::CustomScanState,
    expr: *mut pg_sys::Expr,
) -> i64 {
    // SAFETY: expr comes from the provider-owned DML custom_exprs list; NodeTag
    // dispatch below guards Const/Param casts and evaluation.
    unsafe {
        if expr.is_null() {
            pgrx::error!("EcSpireDistributedScan DML PK expression is null");
        }
        match (*expr.cast::<pg_sys::Node>()).type_ {
            pg_sys::NodeTag::T_Const => {
                let const_expr = &*expr.cast::<pg_sys::Const>();
                if const_expr.constisnull {
                    pgrx::error!("EcSpireDistributedScan DML constant PK is NULL");
                }
                match const_expr.consttype {
                    pg_sys::INT2OID => i64::from(pg_sys::DatumGetInt16(const_expr.constvalue)),
                    pg_sys::INT4OID => i64::from(pg_sys::DatumGetInt32(const_expr.constvalue)),
                    pg_sys::INT8OID => pg_sys::DatumGetInt64(const_expr.constvalue),
                    other => pgrx::error!(
                        "EcSpireDistributedScan DML path unsupported PK type OID {}",
                        other.to_u32()
                    ),
                }
            }
            pg_sys::NodeTag::T_Param => {
                let param = &*expr.cast::<pg_sys::Param>();
                let expr_state = pg_sys::ExecInitExpr(expr, &mut (*node).ss.ps);
                if expr_state.is_null() {
                    pgrx::error!("EcSpireDistributedScan failed to initialize DML PK parameter");
                }
                let eval = (*expr_state).evalfunc.unwrap_or_else(|| {
                    pgrx::error!("EcSpireDistributedScan DML PK parameter has no evaluator")
                });
                let mut is_null = false;
                let datum = eval(expr_state, (*node).ss.ps.ps_ExprContext, &mut is_null);
                if is_null {
                    pgrx::error!("EcSpireDistributedScan DML PK parameter must not be NULL");
                }
                match param.paramtype {
                    pg_sys::INT2OID => i64::from(pg_sys::DatumGetInt16(datum)),
                    pg_sys::INT4OID => i64::from(pg_sys::DatumGetInt32(datum)),
                    pg_sys::INT8OID => pg_sys::DatumGetInt64(datum),
                    other => pgrx::error!(
                        "EcSpireDistributedScan DML path unsupported PK type OID {}",
                        other.to_u32()
                    ),
                }
            }
            _ => pgrx::error!(
                "EcSpireDistributedScan DML path requires a constant or parameter bigint PK"
            ),
        }
    }
}

fn custom_scan_execute_dml_delete(state: &SpireCustomScanExecState) -> u64 {
    let invocation =
        custom_scan_dml_primitive_invocation(state).unwrap_or_else(|e| pgrx::error!("{e}"));
    if invocation.mode != super::SpireDmlFrontdoorCustomScanMode::CoordinatorDeleteTuplePayload {
        pgrx::error!("EcSpireDistributedScan DML DELETE got non-delete primitive mode");
    }
    let deleted_count = Spi::connect(|client| {
        client
            .select(
                "SELECT remote_deleted_count \
                   FROM ec_spire_prepare_coordinator_delete_tuple_payload(\
                        $1::oid, $2::text, $3::bytea)",
                None,
                &[
                    invocation.index_oid.into(),
                    invocation.pk_column.as_str().into(),
                    invocation.pk_value.to_vec().into(),
                ],
            )
            .map_err(|e| format!("EcSpireDistributedScan DML DELETE primitive failed: {e}"))?
            .map(|row| {
                row["remote_deleted_count"]
                    .value::<i64>()
                    .map_err(|e| {
                        format!(
                            "EcSpireDistributedScan DML DELETE deleted_count decode failed: {e}"
                        )
                    })?
                    .ok_or_else(|| {
                        "EcSpireDistributedScan DML DELETE deleted_count is null".to_owned()
                    })
            })
            .next()
            .transpose()
            .map(|value| {
                value.ok_or_else(|| {
                    "EcSpireDistributedScan DML DELETE primitive returned no rows".to_owned()
                })
            })?
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    u64::try_from(deleted_count).unwrap_or_else(|_| {
        pgrx::error!("EcSpireDistributedScan DML DELETE returned a negative deleted_count")
    })
}

unsafe fn custom_scan_execute_dml_update(
    state: &SpireCustomScanExecState,
    scan_state: *mut pg_sys::ScanState,
) -> u64 {
    let invocation =
        custom_scan_dml_primitive_invocation(state).unwrap_or_else(|e| pgrx::error!("{e}"));
    if invocation.mode != super::SpireDmlFrontdoorCustomScanMode::CoordinatorUpdateTuplePayload {
        pgrx::error!("EcSpireDistributedScan DML UPDATE got non-update primitive mode");
    }
    if state.dml_update_value_exprs.len() != invocation.updated_columns.len() {
        pgrx::error!(
            "EcSpireDistributedScan DML UPDATE value expression count does not match updated columns"
        );
    }
    let row_payload_json = unsafe {
        let node = scan_state.cast::<pg_sys::CustomScanState>();
        let mut payload = serde_json::Map::with_capacity(invocation.updated_columns.len());
        for (column, expr) in invocation
            .updated_columns
            .iter()
            .zip(state.dml_update_value_exprs.iter().copied())
        {
            let value = custom_scan_dml_update_expr_json_value(node, expr);
            payload.insert(column.clone(), value);
        }
        serde_json::Value::Object(payload).to_string()
    };
    let updated_column_refs = invocation
        .updated_columns
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let updated_count = Spi::connect(|client| {
        client
            .select(
                "SELECT remote_updated_count \
                   FROM ec_spire_forward_coordinator_update_tuple_payload(\
                        $1::oid, $2::text, $3::bytea, $4::jsonb, $5::text[])",
                None,
                &[
                    invocation.index_oid.into(),
                    invocation.pk_column.as_str().into(),
                    invocation.pk_value.to_vec().into(),
                    row_payload_json.as_str().into(),
                    updated_column_refs.as_slice().into(),
                ],
            )
            .map_err(|e| format!("EcSpireDistributedScan DML UPDATE primitive failed: {e}"))?
            .map(|row| {
                row["remote_updated_count"]
                    .value::<i64>()
                    .map_err(|e| {
                        format!(
                            "EcSpireDistributedScan DML UPDATE updated_count decode failed: {e}"
                        )
                    })?
                    .ok_or_else(|| {
                        "EcSpireDistributedScan DML UPDATE updated_count is null".to_owned()
                    })
            })
            .next()
            .transpose()
            .map(|value| {
                value.ok_or_else(|| {
                    "EcSpireDistributedScan DML UPDATE primitive returned no rows".to_owned()
                })
            })?
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    u64::try_from(updated_count).unwrap_or_else(|_| {
        pgrx::error!("EcSpireDistributedScan DML UPDATE returned a negative updated_count")
    })
}

unsafe fn custom_scan_dml_update_expr_json_value(
    node: *mut pg_sys::CustomScanState,
    expr: *mut pg_sys::Expr,
) -> serde_json::Value {
    // SAFETY: expr is a provider-owned UPDATE value expression; NodeTag dispatch
    // below guards Const/Param casts and evaluation. Non-null Datum values are
    // immediately converted through PostgreSQL's type output function.
    unsafe {
        let datum_json_value = |datum: pg_sys::Datum, typoid: pg_sys::Oid| {
            if typoid == pg_sys::InvalidOid {
                pgrx::error!("EcSpireDistributedScan DML UPDATE value has invalid type OID");
            }
            let mut typoutput = pg_sys::InvalidOid;
            let mut typisvarlena = false;
            pg_sys::getTypeOutputInfo(typoid, &mut typoutput, &mut typisvarlena);
            let mut flinfo = std::mem::MaybeUninit::<pg_sys::FmgrInfo>::zeroed().assume_init();
            pg_sys::fmgr_info(typoutput, &mut flinfo);
            let output = pg_sys::OutputFunctionCall(&mut flinfo, datum);
            if output.is_null() {
                pgrx::error!("EcSpireDistributedScan DML UPDATE type output returned NULL");
            }
            let value = std::ffi::CStr::from_ptr(output)
                .to_str()
                .unwrap_or_else(|_| {
                    pgrx::error!("EcSpireDistributedScan DML UPDATE output value is not UTF-8")
                })
                .to_owned();
            serde_json::Value::String(value)
        };
        if expr.is_null() {
            pgrx::error!("EcSpireDistributedScan DML UPDATE value expression is null");
        }
        match (*expr.cast::<pg_sys::Node>()).type_ {
            pg_sys::NodeTag::T_Const => {
                let const_expr = &*expr.cast::<pg_sys::Const>();
                if const_expr.constisnull {
                    serde_json::Value::Null
                } else {
                    datum_json_value(const_expr.constvalue, const_expr.consttype)
                }
            }
            pg_sys::NodeTag::T_Param => {
                let expr_state = pg_sys::ExecInitExpr(expr, &mut (*node).ss.ps);
                if expr_state.is_null() {
                    pgrx::error!(
                        "EcSpireDistributedScan failed to initialize DML UPDATE parameter"
                    );
                }
                let eval = (*expr_state).evalfunc.unwrap_or_else(|| {
                    pgrx::error!("EcSpireDistributedScan DML UPDATE parameter has no evaluator")
                });
                let mut is_null = false;
                let datum = eval(expr_state, (*node).ss.ps.ps_ExprContext, &mut is_null);
                if is_null {
                    serde_json::Value::Null
                } else {
                    let typoid = pg_sys::exprType(expr.cast());
                    datum_json_value(datum, typoid)
                }
            }
            _ => pgrx::error!(
                "EcSpireDistributedScan DML UPDATE supports only constant or parameter SET values in v1"
            ),
        }
    }
}

fn custom_scan_ensure_dml_pk_select_payload(state: &mut SpireCustomScanExecState) {
    if state.dml_payload_loaded {
        return;
    }
    let invocation =
        custom_scan_dml_primitive_invocation(state).unwrap_or_else(|e| pgrx::error!("{e}"));
    if invocation.mode != super::SpireDmlFrontdoorCustomScanMode::CoordinatorPkSelectTuplePayload {
        pgrx::error!("EcSpireDistributedScan DML PK SELECT got non-select primitive mode");
    }
    let requested_columns =
        custom_scan_dml_pk_select_requested_columns(&invocation, &state.tuple_payload_columns);
    let requested_column_refs = requested_columns
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let tuple_payload_json = Spi::connect(|client| {
        client
            .select(
                "SELECT selected_count, tuple_payload_json \
                   FROM ec_spire_forward_coordinator_select_tuple_payload(\
                        $1::oid, $2::text, $3::bytea, $4::text[])",
                None,
                &[
                    invocation.index_oid.into(),
                    invocation.pk_column.as_str().into(),
                    invocation.pk_value.to_vec().into(),
                    requested_column_refs.as_slice().into(),
                ],
            )
            .map_err(|e| format!("EcSpireDistributedScan DML PK SELECT primitive failed: {e}"))?
            .map(|row| {
                let selected_count = row["selected_count"]
                    .value::<i64>()
                    .map_err(|e| {
                        format!(
                            "EcSpireDistributedScan DML PK SELECT selected_count decode failed: {e}"
                        )
                    })?
                    .ok_or_else(|| {
                        "EcSpireDistributedScan DML PK SELECT selected_count is null".to_owned()
                    })?;
                let payload = row["tuple_payload_json"].value::<String>().map_err(|e| {
                    format!("EcSpireDistributedScan DML PK SELECT payload decode failed: {e}")
                })?;
                Ok::<Option<String>, String>((selected_count == 1).then_some(payload).flatten())
            })
            .next()
            .transpose()
            .map(|value| {
                value.ok_or_else(|| {
                    "EcSpireDistributedScan DML PK SELECT primitive returned no rows".to_owned()
                })
            })?
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    state.dml_tuple_payload_json = tuple_payload_json;
    state.dml_payload_loaded = true;
}

fn custom_scan_dml_pk_select_requested_columns(
    invocation: &super::dml_frontdoor::SpireDmlFrontdoorPrimitiveInvocation,
    tuple_payload_columns: &[String],
) -> Vec<String> {
    let mut columns = if tuple_payload_columns.is_empty() {
        invocation.projected_columns.clone()
    } else {
        tuple_payload_columns.to_vec()
    };
    if !columns.iter().any(|column| column == &invocation.pk_column) {
        columns.insert(0, invocation.pk_column.clone());
    }
    columns
}

fn custom_scan_ensure_outputs(state: &mut SpireCustomScanExecState) {
    if state.loaded_outputs {
        return;
    }
    let index_oid = state.index_oid;
    let query = state.query.clone();
    let top_k = state.top_k;
    let tuple_payload_columns = state.tuple_payload_columns.clone();
    let index_relation = crate::storage::relation_guard::IndexRelationGuard::access_share(
        index_oid,
        "EcSpireDistributedScan production executor",
    );
    let stream = super::remote_search_production_scan_tuple_payload_result_stream(
        index_relation.as_ptr(),
        query,
        top_k,
        &tuple_payload_columns,
    );
    if stream.summary.next_blocker != super::SPIRE_REMOTE_NONE {
        pgrx::error!(
            "EcSpireDistributedScan production executor blocked: status {}, next_blocker {}, recommendation {}",
            stream.summary.status,
            stream.summary.next_blocker,
            stream.summary.recommendation
        );
    }
    state.outputs = stream.outputs;
    state.loaded_outputs = true;
}
