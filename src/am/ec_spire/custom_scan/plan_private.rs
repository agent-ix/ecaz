#[derive(Clone, Copy)]
struct CustomScanPlanPrivate<'a> {
    list: *mut pg_sys::List,
    len: i32,
    tag: pg_sys::NodeTag,
    _list: std::marker::PhantomData<&'a pg_sys::List>,
}

impl<'a> CustomScanPlanPrivate<'a> {
    unsafe fn new(list: *mut pg_sys::List) -> Option<Self> {
        // SAFETY: caller guarantees `list`, when non-null, is a provider-owned
        // PostgreSQL List that remains live for immediate plan-private reads.
        let list_ref = unsafe { custom_scan_pg_ref(list)? };
        Some(Self {
            list,
            len: list_ref.length,
            tag: list_ref.type_,
            _list: std::marker::PhantomData,
        })
    }

    fn has_offset(self, offset: i32) -> bool {
        offset >= 0 && offset < self.len
    }

    fn require_offset(self, offset: i32, label: &str) {
        if !self.has_offset(offset) {
            pgrx::error!("EcSpireDistributedScan plan is missing private {label}");
        }
    }

    fn nth_oid(self, offset: i32) -> Option<pg_sys::Oid> {
        if !self.has_offset(offset) {
            return None;
        }
        // SAFETY: the list is live and `offset` is bounds-checked against the
        // PostgreSQL List length captured at construction.
        Some(unsafe { pg_sys::list_nth_oid(self.list, offset) })
    }

    fn nth_node(self, offset: i32) -> Option<*mut pg_sys::Node> {
        if !self.has_offset(offset) {
            return None;
        }
        // SAFETY: the list is live and `offset` is bounds-checked against the
        // PostgreSQL List length captured at construction.
        Some(unsafe { pg_sys::list_nth(self.list, offset).cast::<pg_sys::Node>() })
    }

    fn string_node_value(self, node: *mut pg_sys::Node, label: &str) -> String {
        // SAFETY: node is checked for null/String tag and sval null before
        // reading the PostgreSQL-owned NUL-terminated string.
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

    fn u32_at(self, offset: i32, label: &str) -> u32 {
        self.require_offset(offset, label);
        match self.tag {
            pg_sys::NodeTag::T_OidList => self
                .nth_oid(offset)
                .unwrap_or_else(|| {
                    pgrx::error!("EcSpireDistributedScan plan is missing private {label}")
                })
                .to_u32(),
            pg_sys::NodeTag::T_List => {
                let node = self.nth_node(offset).unwrap_or_else(|| {
                    pgrx::error!("EcSpireDistributedScan plan has null private {label}")
                });
                // SAFETY: `node` came from the live provider-owned
                // plan-private List at a bounds-checked offset.
                let Some(node_ref) = (unsafe { custom_scan_pg_ref(node) }) else {
                    pgrx::error!("EcSpireDistributedScan plan has null private {label}");
                };
                match node_ref.type_ {
                    pg_sys::NodeTag::T_Integer => {
                        // SAFETY: the node tag proves this List item is an
                        // Integer node before the cast.
                        let Some(value_node) =
                            (unsafe { custom_scan_pg_ref(node.cast::<pg_sys::Integer>()) })
                        else {
                            pgrx::error!("EcSpireDistributedScan plan has null private {label}");
                        };
                        u32::try_from(value_node.ival).unwrap_or_else(|_| {
                            pgrx::error!("EcSpireDistributedScan plan private {label} is negative")
                        })
                    }
                    pg_sys::NodeTag::T_String => self
                        .string_node_value(node, label)
                        .parse::<u32>()
                        .unwrap_or_else(|e| {
                            pgrx::error!(
                                "EcSpireDistributedScan plan private {label} is not u32: {e}"
                            )
                        }),
                    _ => pgrx::error!("EcSpireDistributedScan plan has invalid private {label}"),
                }
            }
            _ => pgrx::error!("EcSpireDistributedScan plan has invalid private metadata list"),
        }
    }
}

#[derive(Clone, Copy)]
struct CustomScanPlan<'a> {
    plan: *mut pg_sys::CustomScan,
    plan_ref: &'a pg_sys::CustomScan,
}

impl<'a> CustomScanPlan<'a> {
    unsafe fn new(plan: *mut pg_sys::CustomScan) -> Option<Self> {
        // SAFETY: caller guarantees `plan` is the provider-owned CustomScan
        // node for immediate inspection during the current PostgreSQL callback.
        let plan_ref = unsafe { custom_scan_pg_ref(plan)? };
        Some(Self { plan, plan_ref })
    }

    unsafe fn from_state(node: *mut pg_sys::CustomScanState) -> Self {
        // SAFETY: caller guarantees node is the live CustomScanState for the
        // current executor/explain callback.
        let Some(state) = (unsafe { custom_scan_pg_ref(node) }) else {
            pgrx::error!("EcSpireDistributedScan executor state is NULL");
        };
        unsafe { Self::new(state.ss.ps.plan.cast::<pg_sys::CustomScan>()) }
            .unwrap_or_else(|| pgrx::error!("EcSpireDistributedScan plan is NULL"))
    }

    fn as_ptr(self) -> *mut pg_sys::CustomScan {
        self.plan
    }

    fn custom_private(self, label: &str) -> CustomScanPlanPrivate<'a> {
        if self.plan_ref.custom_private.is_null() {
            pgrx::error!("EcSpireDistributedScan plan is missing {label} metadata");
        }
        // SAFETY: this view was constructed from the live provider-owned
        // CustomScan plan for the current callback; custom_private is checked
        // for null before the list wrapper is built.
        unsafe { CustomScanPlanPrivate::new(self.plan_ref.custom_private) }
            .unwrap_or_else(|| pgrx::error!("EcSpireDistributedScan plan is missing {label} metadata"))
    }

    fn mode(self) -> SpireCustomScanPlanMode {
        let custom_private = self.custom_private("private mode");
        let raw = custom_private.u32_at(0, "mode");
        custom_scan_mode_from_u32(raw)
            .unwrap_or_else(|| pgrx::error!("EcSpireDistributedScan plan has unknown mode {raw}"))
    }

    fn index_oid(self) -> pg_sys::Oid {
        let custom_private = self.custom_private("private index OID");
        pg_sys::Oid::from(custom_private.u32_at(1, "index OID"))
    }

    fn dml_column_list(self, offset: i32, label: &str) -> Vec<String> {
        let custom_private = self.custom_private(label);
        custom_scan_dml_column_list_from_plan_private(custom_private, offset, label)
    }

    fn dml_pk_column(self) -> String {
        let custom_private = self.custom_private("PK column");
        custom_scan_dml_pk_column_from_plan_private(custom_private)
    }
}

unsafe fn custom_scan_expr_is_query_value(expr: *mut pg_sys::Expr) -> bool {
    // SAFETY: caller guarantees expr is a live planner expression for
    // immediate node-tag dispatch and Const/Param inspection.
    match unsafe { custom_scan_expr_node_tag(expr) } {
        Some(pg_sys::NodeTag::T_Const) => unsafe {
            custom_scan_query_values_from_const(expr.cast()).is_some()
        },
        Some(pg_sys::NodeTag::T_Param) => {
            unsafe { custom_scan_pg_ref(expr.cast::<pg_sys::Param>()) }
                .map(|param| param.paramtype == pg_sys::FLOAT4ARRAYOID)
                .unwrap_or(false)
        }
        _ => false,
    }
}

unsafe fn custom_scan_query_values_from_const(const_expr: *mut pg_sys::Const) -> Option<Vec<f32>> {
    // SAFETY: caller guarantees const_expr is a live planner Const node.
    let const_ref = unsafe { custom_scan_pg_ref(const_expr)? };
    // SAFETY: null was checked above and caller only passes PostgreSQL Const
    // nodes from planner expressions. Float4 array type/nullability are checked
    // before decoding the datum without taking ownership from PostgreSQL.
    let values = unsafe {
        if const_ref.constisnull || const_ref.consttype != pg_sys::FLOAT4ARRAYOID {
            return None;
        }
        Vec::<f32>::from_polymorphic_datum(const_ref.constvalue, false, pg_sys::FLOAT4ARRAYOID)?
    };
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return None;
    }
    Some(values)
}

unsafe fn custom_scan_mode_from_path(
    custom_path: *mut pg_sys::CustomPath,
) -> Option<SpireCustomScanPlanMode> {
    // SAFETY: caller guarantees custom_path is a live CustomPath produced by
    // this provider for immediate private-list inspection.
    let custom_private = unsafe { custom_scan_pg_ref(custom_path)? }.custom_private;
    let custom_private = unsafe { CustomScanPlanPrivate::new(custom_private)? };
    custom_scan_mode_from_u32(custom_private.nth_oid(0)?.to_u32())
}

unsafe fn custom_scan_index_oid_from_path(custom_path: *mut pg_sys::CustomPath) -> pg_sys::Oid {
    // SAFETY: caller guarantees custom_path is a live CustomPath produced by
    // this provider for immediate private-list inspection.
    let Some(custom_path) = (unsafe { custom_scan_pg_ref(custom_path) }) else {
        pgrx::error!("EcSpireDistributedScan CustomPath is missing private index OID");
    };
    let custom_private = unsafe { CustomScanPlanPrivate::new(custom_path.custom_private) }
        .unwrap_or_else(|| {
            pgrx::error!("EcSpireDistributedScan CustomPath is missing private index OID")
        });
    custom_private.nth_oid(1).unwrap_or_else(|| {
        pgrx::error!("EcSpireDistributedScan CustomPath is missing private index OID")
    })
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

unsafe fn custom_scan_dml_plan_private(
    mode: SpireCustomScanPlanMode,
    index_oid: pg_sys::Oid,
    pk_column: &str,
    updated_columns: &[String],
    projected_columns: &[String],
) -> *mut pg_sys::List {
    // SAFETY: caller guarantees PostgreSQL planner memory context is active.
    let mut custom_private = unsafe { CustomScanPlanPrivateBuilder::new() };
    custom_private.append_string(&mode.raw().to_string());
    custom_private.append_string(&index_oid.to_u32().to_string());
    custom_private.append_counted_column_list(updated_columns);
    custom_private.append_counted_column_list(projected_columns);
    custom_private.append_string(pk_column);
    custom_private.into_pg()
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn custom_scan_dml_plan_private_copy_roundtrip_for_test() -> String {
    // SAFETY: test helper builds provider-owned plan-private metadata and asks
    // PostgreSQL to deep-copy it before reading it back.
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
        let copied = CustomScanPlanPrivate::new(copied)
            .unwrap_or_else(|| pgrx::error!("copied plan-private list is NULL"));
        let mode = copied.u32_at(0, "mode");
        let index_oid = copied.u32_at(1, "index OID");
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

fn custom_scan_plan_private_u32(
    custom_private: CustomScanPlanPrivate<'_>,
    offset: i32,
    label: &str,
) -> u32 {
    custom_private.u32_at(offset, label)
}

struct CustomScanPlanPrivateBuilder {
    list: *mut pg_sys::List,
}

impl CustomScanPlanPrivateBuilder {
    unsafe fn new() -> Self {
        Self {
            list: std::ptr::null_mut(),
        }
    }

    fn append_string(&mut self, value: &str) {
        // SAFETY: this builder is only constructed while PostgreSQL planner
        // memory is active. CString rejects interior NULs, and pstrdup copies
        // into PostgreSQL memory before makeString/lappend attach it.
        unsafe {
            let c_value = CString::new(value).unwrap_or_else(|_| {
                pgrx::error!("EcSpireDistributedScan plan-private string contains NUL")
            });
            let copied = pg_sys::pstrdup(c_value.as_ptr());
            self.list = pg_sys::lappend(self.list, pg_sys::makeString(copied).cast());
        }
    }

    fn append_counted_column_list(&mut self, columns: &[String]) {
        self.append_string(&columns.len().to_string());
        for column in columns {
            self.append_string(column);
        }
    }

    fn into_pg(self) -> *mut pg_sys::List {
        self.list
    }
}

fn custom_scan_dml_column_list_from_plan_private(
    custom_private: CustomScanPlanPrivate<'_>,
    offset: i32,
    label: &str,
) -> Vec<String> {
    if !custom_private.has_offset(offset) {
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

fn custom_scan_dml_counted_column_list_from_plan_private(
    custom_private: CustomScanPlanPrivate<'_>,
    count_offset: i32,
    label: &str,
) -> Result<Vec<String>, String> {
    if !custom_private.has_offset(count_offset) {
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
    if custom_private.len < required_len {
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
        let column = custom_private.string_node_value(
            custom_private.nth_node(offset).unwrap_or_else(|| {
                pgrx::error!("EcSpireDistributedScan DML plan {label} metadata is truncated")
            }),
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

fn custom_scan_dml_projected_column_count_offset(custom_private: CustomScanPlanPrivate<'_>) -> i32 {
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

fn custom_scan_dml_pk_column_offset(custom_private: CustomScanPlanPrivate<'_>) -> i32 {
    let projected_offset = custom_scan_dml_projected_column_count_offset(custom_private);
    let projected_count =
        custom_scan_plan_private_u32(custom_private, projected_offset, "projected column count");
    projected_offset
        + 1
        + i32::try_from(projected_count).unwrap_or_else(|_| {
            pgrx::error!("EcSpireDistributedScan DML plan projected column count is too large")
        })
}

fn custom_scan_dml_pk_column_from_plan_private(
    custom_private: CustomScanPlanPrivate<'_>,
) -> String {
    let offset = custom_scan_dml_pk_column_offset(custom_private);
    let node = custom_private.nth_node(offset).unwrap_or_else(|| {
        pgrx::error!("EcSpireDistributedScan DML plan is missing PK column metadata")
    });
    let pk_column = custom_private.string_node_value(node, "PK column");
    if pk_column.is_empty() {
        pgrx::error!("EcSpireDistributedScan DML plan PK column metadata is empty");
    }
    pk_column
}
