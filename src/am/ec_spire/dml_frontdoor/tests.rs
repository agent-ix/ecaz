#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifier_accepts_update_delete_and_pk_select_v1_shapes() {
        assert_eq!(
            classify_dml_frontdoor_shape(update_input(&["title"], &["embedding"])).kind,
            "update_non_embedding_by_pk"
        );
        assert_eq!(
            classify_dml_frontdoor_shape(delete_input()).kind,
            "delete_by_pk"
        );
        assert_eq!(
            classify_dml_frontdoor_shape(select_input(&["id", "title"])).kind,
            "pk_select_by_pk"
        );
    }

    #[test]
    fn classifier_rejects_joins_subqueries_and_returning() {
        let mut joined = update_input(&["title"], &["embedding"]);
        joined.has_join = true;
        assert_eq!(
            classify_dml_frontdoor_shape(joined).kind,
            "unsupported_join_shape"
        );

        let mut subquery = delete_input();
        subquery.has_subquery = true;
        assert_eq!(
            classify_dml_frontdoor_shape(subquery).kind,
            "unsupported_subquery_shape"
        );

        let mut returning = delete_input();
        returning.has_returning = true;
        assert_eq!(
            classify_dml_frontdoor_shape(returning).kind,
            "unsupported_returning_shape"
        );
    }

    #[test]
    fn classifier_requires_bigint_pk_equality_predicate() {
        let mut wrong_column = select_input(&["id"]);
        wrong_column.predicate_column = Some("title");
        assert_eq!(
            classify_dml_frontdoor_shape(wrong_column).kind,
            "non_pk_select_pass_through"
        );

        let mut wrong_operator = select_input(&["id"]);
        wrong_operator.predicate_operator = Some(">");
        assert_eq!(
            classify_dml_frontdoor_shape(wrong_operator).kind,
            "unsupported_pk_predicate"
        );

        let mut wrong_value = select_input(&["id"]);
        wrong_value.predicate_value_kind = SpireDmlFrontdoorValueKind::Other;
        assert_eq!(
            classify_dml_frontdoor_shape(wrong_value).kind,
            "unsupported_pk_predicate"
        );
    }

    #[test]
    fn classifier_allows_non_pk_selects_to_pass_through() {
        let mut no_predicate = select_input(&["id"]);
        no_predicate.predicate_column = None;
        no_predicate.predicate_operator = None;
        no_predicate.predicate_value_kind = SpireDmlFrontdoorValueKind::Other;
        let no_predicate_shape = classify_dml_frontdoor_shape(no_predicate);
        assert!(!no_predicate_shape.supported);
        assert_eq!(no_predicate_shape.kind, "non_pk_select_pass_through");
        assert_eq!(no_predicate_shape.hint, None);

        let mut non_pk_predicate = select_input(&["id"]);
        non_pk_predicate.predicate_column = Some("title");
        non_pk_predicate.predicate_value_kind = SpireDmlFrontdoorValueKind::Other;
        assert_eq!(
            classify_dml_frontdoor_shape(non_pk_predicate).kind,
            "non_pk_select_pass_through"
        );
    }

    #[test]
    fn classifier_rejects_embedding_and_pk_updates() {
        assert_eq!(
            classify_dml_frontdoor_shape(update_input(&["embedding"], &["embedding"])).kind,
            "embedding_update_rejected"
        );
        assert_eq!(
            classify_dml_frontdoor_shape(update_input(&["id"], &["embedding"])).kind,
            "unsupported_pk_update"
        );
    }

    #[test]
    fn classifier_rejects_empty_update_or_projection() {
        assert_eq!(
            classify_dml_frontdoor_shape(update_input(&[], &["embedding"])).kind,
            "unsupported_empty_update"
        );
        assert_eq!(
            classify_dml_frontdoor_shape(select_input(&[])).kind,
            "unsupported_empty_projection"
        );
    }

    #[test]
    fn query_layer_maps_command_and_subquery_flags() {
        let mut update_query = pg_sys::Query::default();
        update_query.commandType = pg_sys::CmdType::CMD_UPDATE;
        assert_eq!(
            dml_frontdoor_operation_for_query(&update_query),
            Some(SpireDmlFrontdoorOperation::Update)
        );

        let mut delete_query = pg_sys::Query::default();
        delete_query.commandType = pg_sys::CmdType::CMD_DELETE;
        assert_eq!(
            dml_frontdoor_operation_for_query(&delete_query),
            Some(SpireDmlFrontdoorOperation::Delete)
        );

        let mut select_query = pg_sys::Query::default();
        select_query.commandType = pg_sys::CmdType::CMD_SELECT;
        assert_eq!(
            dml_frontdoor_operation_for_query(&select_query),
            Some(SpireDmlFrontdoorOperation::PkSelect)
        );
        assert!(!dml_frontdoor_query_has_subquery_shape(&select_query));

        select_query.hasSubLinks = true;
        assert!(dml_frontdoor_query_has_subquery_shape(&select_query));
    }

    #[test]
    fn baserel_handoff_uses_only_target_rel_for_dml() {
        let mut query = pg_sys::Query::default();
        query.resultRelation = 1;

        let mut rel = pg_sys::RelOptInfo::default();
        rel.relid = 2;
        assert_eq!(
            dml_frontdoor_baserel_target_rtindex(&query, &rel, SpireDmlFrontdoorOperation::Update)
                .unwrap(),
            None
        );

        rel.relid = 1;
        assert_eq!(
            dml_frontdoor_baserel_target_rtindex(&query, &rel, SpireDmlFrontdoorOperation::Delete)
                .unwrap(),
            Some(1)
        );

        rel.relid = 2;
        assert_eq!(
            dml_frontdoor_baserel_target_rtindex(
                &query,
                &rel,
                SpireDmlFrontdoorOperation::PkSelect
            )
            .unwrap(),
            Some(2)
        );
    }

    #[test]
    fn baserel_primitive_plan_mode_guard_names_expected_operation() {
        let plan_expr = test_primitive_plan_expr(
            SpireDmlFrontdoorCustomScanMode::CoordinatorDeleteTuplePayload,
        );
        let error = match dml_frontdoor_primitive_plan_expr_require_mode(
            plan_expr,
            SpireDmlFrontdoorCustomScanMode::CoordinatorUpdateTuplePayload,
            "UPDATE",
        ) {
            Ok(_plan_expr) => panic!("mismatched DML primitive mode should fail"),
            Err(error) => error,
        };

        assert_eq!(
            error,
            "ec_spire DML frontdoor baserel expression handoff expected UPDATE primitive plan"
        );

        let plan_expr = test_primitive_plan_expr(
            SpireDmlFrontdoorCustomScanMode::CoordinatorUpdateTuplePayload,
        );
        assert!(dml_frontdoor_primitive_plan_expr_require_mode(
            plan_expr,
            SpireDmlFrontdoorCustomScanMode::CoordinatorUpdateTuplePayload,
            "UPDATE",
        )
        .is_ok());
    }

    #[test]
    fn query_layer_recognizes_bigint_const_and_param_values() {
        let mut bigint_const = pg_sys::Const::default();
        bigint_const.xpr.type_ = pg_sys::NodeTag::T_Const;
        bigint_const.consttype = pg_sys::INT8OID;
        assert_eq!(
            unsafe {
                dml_frontdoor_value_kind(
                    (&mut bigint_const as *mut pg_sys::Const).cast::<pg_sys::Expr>(),
                )
            },
            SpireDmlFrontdoorValueKind::ConstBigint
        );

        bigint_const.constisnull = true;
        assert_eq!(
            unsafe {
                dml_frontdoor_value_kind(
                    (&mut bigint_const as *mut pg_sys::Const).cast::<pg_sys::Expr>(),
                )
            },
            SpireDmlFrontdoorValueKind::Other
        );

        let mut bigint_param = pg_sys::Param::default();
        bigint_param.xpr.type_ = pg_sys::NodeTag::T_Param;
        bigint_param.paramtype = pg_sys::INT8OID;
        assert_eq!(
            unsafe {
                dml_frontdoor_value_kind(
                    (&mut bigint_param as *mut pg_sys::Param).cast::<pg_sys::Expr>(),
                )
            },
            SpireDmlFrontdoorValueKind::ParamBigint
        );
    }

    #[test]
    fn query_layer_rejects_float_and_numeric_pk_predicate_values() {
        for consttype in [
            pg_sys::FLOAT4OID,
            pg_sys::FLOAT8OID,
            pg_sys::NUMERICOID,
        ] {
            let mut const_expr = pg_sys::Const::default();
            const_expr.xpr.type_ = pg_sys::NodeTag::T_Const;
            const_expr.consttype = consttype;
            assert_eq!(
                unsafe {
                    dml_frontdoor_value_kind(
                        (&mut const_expr as *mut pg_sys::Const).cast::<pg_sys::Expr>(),
                    )
                },
                SpireDmlFrontdoorValueKind::Other
            );
        }

        for paramtype in [
            pg_sys::FLOAT4OID,
            pg_sys::FLOAT8OID,
            pg_sys::NUMERICOID,
        ] {
            let mut param = pg_sys::Param::default();
            param.xpr.type_ = pg_sys::NodeTag::T_Param;
            param.paramtype = paramtype;
            assert_eq!(
                unsafe {
                    dml_frontdoor_value_kind(
                        (&mut param as *mut pg_sys::Param).cast::<pg_sys::Expr>(),
                    )
                },
                SpireDmlFrontdoorValueKind::Other
            );
        }
    }

    #[test]
    fn query_layer_walks_nested_integer_coercion_wrappers() {
        let mut int_param = pg_sys::Param::default();
        int_param.xpr.type_ = pg_sys::NodeTag::T_Param;
        int_param.paramtype = pg_sys::INT4OID;

        let mut coerce = pg_sys::CoerceViaIO::default();
        coerce.xpr.type_ = pg_sys::NodeTag::T_CoerceViaIO;
        coerce.resulttype = pg_sys::INT8OID;
        coerce.arg = (&mut int_param as *mut pg_sys::Param).cast::<pg_sys::Expr>();

        let mut relabel = pg_sys::RelabelType::default();
        relabel.xpr.type_ = pg_sys::NodeTag::T_RelabelType;
        relabel.resulttype = pg_sys::INT8OID;
        relabel.arg = (&mut coerce as *mut pg_sys::CoerceViaIO).cast::<pg_sys::Expr>();

        assert_eq!(
            unsafe {
                dml_frontdoor_value_kind(
                    (&mut relabel as *mut pg_sys::RelabelType).cast::<pg_sys::Expr>(),
                )
            },
            SpireDmlFrontdoorValueKind::ParamBigint
        );

        relabel.resulttype = pg_sys::INT4OID;
        assert_eq!(
            unsafe {
                dml_frontdoor_value_kind(
                    (&mut relabel as *mut pg_sys::RelabelType).cast::<pg_sys::Expr>(),
                )
            },
            SpireDmlFrontdoorValueKind::Other
        );
    }

    #[test]
    fn query_layer_recognizes_bigint_integer_equality_variants() {
        for opcode in [
            pg_sys::F_INT8EQ,
            pg_sys::F_INT84EQ,
            pg_sys::F_INT82EQ,
            pg_sys::F_INT48EQ,
            pg_sys::F_INT28EQ,
        ] {
            assert!(dml_frontdoor_bigint_equality_opcode(pg_sys::Oid::from(
                opcode
            )));
        }
        assert!(!dml_frontdoor_bigint_equality_opcode(pg_sys::Oid::from(
            pg_sys::F_INT4EQ
        )));
    }

    #[test]
    fn query_layer_binds_target_relation_var_to_column_name() {
        let context = SpireDmlFrontdoorQueryContext {
            ec_spire_distributed_table: true,
            pk_column: "id",
            column_names: &[(1, "id"), (2, "title"), (3, "embedding")],
            embedding_columns: &["embedding"],
        };
        let mut var = pg_sys::Var::default();
        var.xpr.type_ = pg_sys::NodeTag::T_Var;
        var.varno = 1;
        var.varattno = 1;

        assert_eq!(
            unsafe {
                dml_frontdoor_var_column(
                    (&mut var as *mut pg_sys::Var).cast::<pg_sys::Expr>(),
                    1,
                    &context,
                )
            },
            Some("id".to_owned())
        );

        var.varno = 2;
        assert_eq!(
            unsafe {
                dml_frontdoor_var_column(
                    (&mut var as *mut pg_sys::Var).cast::<pg_sys::Expr>(),
                    1,
                    &context,
                )
            },
            None
        );
    }

    fn update_input<'a>(
        updated_columns: &'a [&'a str],
        embedding_columns: &'a [&'a str],
    ) -> SpireDmlFrontdoorShapeInput<'a> {
        base_input(
            SpireDmlFrontdoorOperation::Update,
            updated_columns,
            &[],
            embedding_columns,
        )
    }

    fn delete_input<'a>() -> SpireDmlFrontdoorShapeInput<'a> {
        base_input(SpireDmlFrontdoorOperation::Delete, &[], &[], &[])
    }

    fn select_input<'a>(projected_columns: &'a [&'a str]) -> SpireDmlFrontdoorShapeInput<'a> {
        base_input(
            SpireDmlFrontdoorOperation::PkSelect,
            &[],
            projected_columns,
            &[],
        )
    }

    fn base_input<'a>(
        operation: SpireDmlFrontdoorOperation,
        updated_columns: &'a [&'a str],
        projected_columns: &'a [&'a str],
        embedding_columns: &'a [&'a str],
    ) -> SpireDmlFrontdoorShapeInput<'a> {
        SpireDmlFrontdoorShapeInput {
            operation,
            ec_spire_distributed_table: true,
            single_table: true,
            has_join: false,
            has_subquery: false,
            has_returning: false,
            pk_column: "id",
            predicate_column: Some("id"),
            predicate_operator: Some("="),
            predicate_value_kind: SpireDmlFrontdoorValueKind::ConstBigint,
            updated_columns,
            projected_columns,
            embedding_columns,
        }
    }

    fn test_primitive_plan_expr(
        mode: SpireDmlFrontdoorCustomScanMode,
    ) -> SpireDmlFrontdoorPrimitivePlanExpr {
        SpireDmlFrontdoorPrimitivePlanExpr {
            primitive_plan: SpireDmlFrontdoorPrimitivePlan {
                index_oid: pg_sys::Oid::from(1),
                mode,
                primitive: dml_frontdoor_primitive_for_mode(mode),
                pk_argument: SpireDmlFrontdoorPkArgument {
                    pk_column: "id".to_owned(),
                    value: SpireDmlFrontdoorPkValuePlan::ConstBigint(1),
                },
                updated_columns: Vec::new(),
                projected_columns: Vec::new(),
            },
            pk_value_expr: std::ptr::null_mut(),
            updated_value_exprs: Vec::new(),
        }
    }
}
