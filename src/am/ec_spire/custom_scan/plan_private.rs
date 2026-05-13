unsafe fn custom_scan_expr_is_query_value(expr: *mut pg_sys::Expr) -> bool {
    if expr.is_null() {
        return false;
    }
    let node = expr.cast::<pg_sys::Node>();
    match unsafe { (*node).type_ } {
        pg_sys::NodeTag::T_Const => unsafe {
            custom_scan_query_values_from_const(expr.cast()).is_some()
        },
        pg_sys::NodeTag::T_Param => unsafe {
            let param = &*expr.cast::<pg_sys::Param>();
            param.paramtype == pg_sys::FLOAT4ARRAYOID
        },
        _ => false,
    }
}

unsafe fn custom_scan_query_values_from_const(const_expr: *mut pg_sys::Const) -> Option<Vec<f32>> {
    if const_expr.is_null() {
        return None;
    }
    let const_ref = unsafe { &*const_expr };
    if const_ref.constisnull || const_ref.consttype != pg_sys::FLOAT4ARRAYOID {
        return None;
    }
    let values = unsafe {
        Vec::<f32>::from_polymorphic_datum(const_ref.constvalue, false, pg_sys::FLOAT4ARRAYOID)?
    };
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return None;
    }
    Some(values)
}

unsafe fn custom_scan_plan(node: *mut pg_sys::CustomScanState) -> *mut pg_sys::CustomScan {
    unsafe { (*node).ss.ps.plan.cast::<pg_sys::CustomScan>() }
}

unsafe fn custom_scan_mode_from_path(
    custom_path: *mut pg_sys::CustomPath,
) -> Option<SpireCustomScanPlanMode> {
    unsafe {
        if custom_path.is_null() || (*custom_path).custom_private.is_null() {
            return None;
        }
        custom_scan_mode_from_u32(pg_sys::list_nth_oid((*custom_path).custom_private, 0).to_u32())
    }
}

unsafe fn custom_scan_index_oid_from_path(custom_path: *mut pg_sys::CustomPath) -> pg_sys::Oid {
    unsafe {
        if custom_path.is_null() || (*custom_path).custom_private.is_null() {
            pgrx::error!("EcSpireDistributedScan CustomPath is missing private index OID");
        }
        pg_sys::list_nth_oid((*custom_path).custom_private, 1)
    }
}

unsafe fn custom_scan_mode_from_plan(
    custom_scan: *mut pg_sys::CustomScan,
) -> SpireCustomScanPlanMode {
    unsafe {
        if custom_scan.is_null() || (*custom_scan).custom_private.is_null() {
            pgrx::error!("EcSpireDistributedScan plan is missing private mode");
        }
        let raw = custom_scan_plan_private_u32((*custom_scan).custom_private, 0, "mode");
        custom_scan_mode_from_u32(raw)
            .unwrap_or_else(|| pgrx::error!("EcSpireDistributedScan plan has unknown mode {raw}"))
    }
}

fn custom_scan_mode_from_u32(raw: u32) -> Option<SpireCustomScanPlanMode> {
    match raw {
        CUSTOM_SCAN_PLAN_MODE_VECTOR_ORDER_LIMIT => Some(SpireCustomScanPlanMode::VectorOrderLimit),
        CUSTOM_SCAN_PLAN_MODE_DML_PK_SELECT => {
            Some(SpireCustomScanPlanMode::DmlPkSelectTuplePayload)
        }
        CUSTOM_SCAN_PLAN_MODE_DML_UPDATE => Some(SpireCustomScanPlanMode::DmlUpdateTuplePayload),
        CUSTOM_SCAN_PLAN_MODE_DML_DELETE => Some(SpireCustomScanPlanMode::DmlDeleteTuplePayload),
        _ => None,
    }
}

fn custom_scan_plan_mode_for_dml_mode(
    mode: super::SpireDmlFrontdoorCustomScanMode,
) -> SpireCustomScanPlanMode {
    match mode {
        super::SpireDmlFrontdoorCustomScanMode::CoordinatorUpdateTuplePayload => {
            SpireCustomScanPlanMode::DmlUpdateTuplePayload
        }
        super::SpireDmlFrontdoorCustomScanMode::CoordinatorDeleteTuplePayload => {
            SpireCustomScanPlanMode::DmlDeleteTuplePayload
        }
        super::SpireDmlFrontdoorCustomScanMode::CoordinatorPkSelectTuplePayload => {
            SpireCustomScanPlanMode::DmlPkSelectTuplePayload
        }
    }
}

unsafe fn custom_scan_index_oid_from_plan(custom_scan: *mut pg_sys::CustomScan) -> pg_sys::Oid {
    unsafe {
        if custom_scan.is_null() || (*custom_scan).custom_private.is_null() {
            pgrx::error!("EcSpireDistributedScan plan is missing private index OID");
        }
        pg_sys::Oid::from(custom_scan_plan_private_u32(
            (*custom_scan).custom_private,
            1,
            "index OID",
        ))
    }
}

unsafe fn custom_scan_dml_plan_private(
    mode: SpireCustomScanPlanMode,
    index_oid: pg_sys::Oid,
    pk_column: &str,
    updated_columns: &[String],
    projected_columns: &[String],
) -> *mut pg_sys::List {
    unsafe {
        let mut custom_private =
            custom_scan_lappend_string(std::ptr::null_mut(), &mode.raw().to_string());
        custom_private =
            custom_scan_lappend_string(custom_private, &index_oid.to_u32().to_string());
        custom_private = custom_scan_lappend_counted_column_list(custom_private, updated_columns);
        custom_private = custom_scan_lappend_counted_column_list(custom_private, projected_columns);
        custom_scan_lappend_string(custom_private, pk_column)
    }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn custom_scan_dml_plan_private_copy_roundtrip_for_test() -> String {
    unsafe {
        let updated_columns = vec!["title".to_owned(), "status".to_owned()];
        let projected_columns = Vec::<String>::new();
        let custom_private = custom_scan_dml_plan_private(
            SpireCustomScanPlanMode::DmlUpdateTuplePayload,
            pg_sys::Oid::from(12_345),
            "id",
            &updated_columns,
            &projected_columns,
        );
        let copied = pg_sys::copyObjectImpl(custom_private.cast()).cast::<pg_sys::List>();
        let mode = custom_scan_plan_private_u32(copied, 0, "mode");
        let index_oid = custom_scan_plan_private_u32(copied, 1, "index OID");
        let updated = custom_scan_dml_column_list_from_plan_private(copied, 2, "updated columns");
        let projected =
            custom_scan_dml_column_list_from_plan_private(copied, 3, "projected columns");
        let pk_column = custom_scan_dml_pk_column_from_plan_private(copied);
        format!(
            "{}|{}|{}|{}|{}",
            mode,
            index_oid,
            updated.join(","),
            projected.join(","),
            pk_column
        )
    }
}

unsafe fn custom_scan_plan_private_u32(
    custom_private: *mut pg_sys::List,
    offset: i32,
    label: &str,
) -> u32 {
    unsafe {
        if custom_private.is_null() || (*custom_private).length <= offset {
            pgrx::error!("EcSpireDistributedScan plan is missing private {label}");
        }
        match (*custom_private).type_ {
            pg_sys::NodeTag::T_OidList => pg_sys::list_nth_oid(custom_private, offset).to_u32(),
            pg_sys::NodeTag::T_List => {
                let node = pg_sys::list_nth(custom_private, offset).cast::<pg_sys::Node>();
                if node.is_null() {
                    pgrx::error!("EcSpireDistributedScan plan has null private {label}");
                }
                match (*node).type_ {
                    pg_sys::NodeTag::T_Integer => {
                        let value = (*node.cast::<pg_sys::Integer>()).ival;
                        u32::try_from(value).unwrap_or_else(|_| {
                            pgrx::error!("EcSpireDistributedScan plan private {label} is negative")
                        })
                    }
                    pg_sys::NodeTag::T_String => custom_scan_string_node_value(node, label)
                        .parse::<u32>()
                        .unwrap_or_else(|e| {
                            pgrx::error!(
                                "EcSpireDistributedScan plan private {label} is not u32: {e}"
                            )
                        }),
                    _ => {
                        pgrx::error!("EcSpireDistributedScan plan has invalid private {label}")
                    }
                }
            }
            _ => pgrx::error!("EcSpireDistributedScan plan has invalid private metadata list"),
        }
    }
}

unsafe fn custom_scan_lappend_string(list: *mut pg_sys::List, value: &str) -> *mut pg_sys::List {
    unsafe {
        let c_value = CString::new(value).unwrap_or_else(|_| {
            pgrx::error!("EcSpireDistributedScan plan-private string contains NUL")
        });
        let copied = pg_sys::pstrdup(c_value.as_ptr());
        pg_sys::lappend(list, pg_sys::makeString(copied).cast())
    }
}

unsafe fn custom_scan_lappend_counted_column_list(
    list: *mut pg_sys::List,
    columns: &[String],
) -> *mut pg_sys::List {
    unsafe {
        let mut list = custom_scan_lappend_string(list, &columns.len().to_string());
        for column in columns {
            list = custom_scan_lappend_string(list, column);
        }
        list
    }
}

unsafe fn custom_scan_string_node_value(node: *mut pg_sys::Node, label: &str) -> String {
    unsafe {
        if node.is_null() || (*node).type_ != pg_sys::NodeTag::T_String {
            pgrx::error!("EcSpireDistributedScan DML plan has invalid {label} metadata");
        }
        let value_node = node.cast::<pg_sys::String>();
        if (*value_node).sval.is_null() {
            pgrx::error!("EcSpireDistributedScan DML plan has null {label} metadata");
        }
        std::ffi::CStr::from_ptr((*value_node).sval)
            .to_str()
            .unwrap_or_else(|_| {
                pgrx::error!("EcSpireDistributedScan DML plan {label} metadata is not UTF-8")
            })
            .to_owned()
    }
}

unsafe fn custom_scan_dml_column_list_from_plan(
    custom_scan: *mut pg_sys::CustomScan,
    offset: i32,
    label: &str,
) -> Vec<String> {
    unsafe {
        if custom_scan.is_null() || (*custom_scan).custom_private.is_null() {
            pgrx::error!("EcSpireDistributedScan DML plan is missing {label} metadata");
        }
        custom_scan_dml_column_list_from_plan_private((*custom_scan).custom_private, offset, label)
    }
}

unsafe fn custom_scan_dml_column_list_from_plan_private(
    custom_private: *mut pg_sys::List,
    offset: i32,
    label: &str,
) -> Vec<String> {
    unsafe {
        if custom_private.is_null() || (*custom_private).length <= offset {
            pgrx::error!("EcSpireDistributedScan DML plan is missing {label} metadata");
        }
        let count_offset = match offset {
            DML_UPDATED_COLUMN_COUNT_OFFSET => DML_UPDATED_COLUMN_COUNT_OFFSET,
            3 => custom_scan_dml_projected_column_count_offset(custom_private),
            _ => pgrx::error!("EcSpireDistributedScan DML plan has invalid {label} offset"),
        };
        custom_scan_dml_counted_column_list_from_plan_private(custom_private, count_offset, label)
            .unwrap_or_else(|e| pgrx::error!("{e}"))
    }
}

unsafe fn custom_scan_dml_counted_column_list_from_plan_private(
    custom_private: *mut pg_sys::List,
    count_offset: i32,
    label: &str,
) -> Result<Vec<String>, String> {
    unsafe {
        if custom_private.is_null() || (*custom_private).length <= count_offset {
            return Err(format!(
                "EcSpireDistributedScan DML plan is missing {label} count metadata"
            ));
        }
        let count = usize::try_from(custom_scan_plan_private_u32(
            custom_private,
            count_offset,
            &format!("{label} count"),
        ))
        .map_err(|_| format!("EcSpireDistributedScan DML plan {label} count is too large"))?;
        let required_len = count_offset
            .checked_add(1)
            .and_then(|offset| offset.checked_add(i32::try_from(count).ok()?))
            .ok_or_else(|| {
                format!("EcSpireDistributedScan DML plan {label} count overflows metadata list")
            })?;
        if (*custom_private).length < required_len {
            return Err(format!(
                "EcSpireDistributedScan DML plan {label} metadata is truncated"
            ));
        }
        let mut columns = Vec::with_capacity(count);
        for index in 0..count {
            let offset = count_offset
                + 1
                + i32::try_from(index).unwrap_or_else(|_| {
                    pgrx::error!("EcSpireDistributedScan DML plan {label} index is too large")
                });
            let column = custom_scan_string_node_value(
                pg_sys::list_nth(custom_private, offset).cast(),
                label,
            );
            if column.is_empty() {
                return Err(format!(
                    "EcSpireDistributedScan DML plan {label} metadata contains an empty column name"
                ));
            }
            columns.push(column);
        }
        Ok(columns)
    }
}

unsafe fn custom_scan_dml_projected_column_count_offset(custom_private: *mut pg_sys::List) -> i32 {
    unsafe {
        let updated_count = custom_scan_plan_private_u32(
            custom_private,
            DML_UPDATED_COLUMN_COUNT_OFFSET,
            "updated column count",
        );
        DML_UPDATED_COLUMN_COUNT_OFFSET
            + 1
            + i32::try_from(updated_count).unwrap_or_else(|_| {
                pgrx::error!("EcSpireDistributedScan DML plan updated column count is too large")
            })
    }
}

unsafe fn custom_scan_dml_pk_column_offset(custom_private: *mut pg_sys::List) -> i32 {
    unsafe {
        let projected_offset = custom_scan_dml_projected_column_count_offset(custom_private);
        let projected_count = custom_scan_plan_private_u32(
            custom_private,
            projected_offset,
            "projected column count",
        );
        projected_offset
            + 1
            + i32::try_from(projected_count).unwrap_or_else(|_| {
                pgrx::error!("EcSpireDistributedScan DML plan projected column count is too large")
            })
    }
}

unsafe fn custom_scan_dml_pk_column_from_plan(custom_scan: *mut pg_sys::CustomScan) -> String {
    unsafe {
        if custom_scan.is_null() || (*custom_scan).custom_private.is_null() {
            pgrx::error!("EcSpireDistributedScan DML plan is missing PK column metadata");
        }
        custom_scan_dml_pk_column_from_plan_private((*custom_scan).custom_private)
    }
}

unsafe fn custom_scan_dml_pk_column_from_plan_private(custom_private: *mut pg_sys::List) -> String {
    unsafe {
        if custom_private.is_null() {
            pgrx::error!("EcSpireDistributedScan DML plan is missing PK column metadata");
        }
        let offset = custom_scan_dml_pk_column_offset(custom_private);
        if (*custom_private).length <= offset {
            pgrx::error!("EcSpireDistributedScan DML plan is missing PK column metadata");
        }
        let pk_column = custom_scan_string_node_value(
            pg_sys::list_nth(custom_private, offset).cast(),
            "PK column",
        );
        if pk_column.is_empty() {
            pgrx::error!("EcSpireDistributedScan DML plan PK column metadata is empty");
        }
        pk_column
    }
}

