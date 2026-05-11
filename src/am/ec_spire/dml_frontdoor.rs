//! ADR-069 DML front-door shape classification.
//!
//! The planner hook maps PostgreSQL query trees into this small input model.
//! Keeping the v1 safety rules here makes unsupported distributed DML shapes
//! fail closed before any hook can fall through to the coordinator heap path.
#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpireDmlFrontdoorOperation {
    Update,
    Delete,
    PkSelect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpireDmlFrontdoorValueKind {
    ConstBigint,
    ParamBigint,
    Other,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SpireDmlFrontdoorShapeInput<'a> {
    pub(crate) operation: SpireDmlFrontdoorOperation,
    pub(crate) ec_spire_distributed_table: bool,
    pub(crate) single_table: bool,
    pub(crate) has_join: bool,
    pub(crate) has_subquery: bool,
    pub(crate) has_returning: bool,
    pub(crate) pk_column: &'a str,
    pub(crate) predicate_column: Option<&'a str>,
    pub(crate) predicate_operator: Option<&'a str>,
    pub(crate) predicate_value_kind: SpireDmlFrontdoorValueKind,
    pub(crate) updated_columns: &'a [&'a str],
    pub(crate) projected_columns: &'a [&'a str],
    pub(crate) embedding_columns: &'a [&'a str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpireDmlFrontdoorShapeRow {
    pub(crate) supported: bool,
    pub(crate) operation: &'static str,
    pub(crate) kind: &'static str,
    pub(crate) status: &'static str,
    pub(crate) error: Option<&'static str>,
    pub(crate) hint: Option<&'static str>,
}

const ADR_069_HINT: &str = "See ADR-069 for the v1 SPIRE distributed DML shape.";

pub(crate) fn classify_dml_frontdoor_shape(
    input: SpireDmlFrontdoorShapeInput<'_>,
) -> SpireDmlFrontdoorShapeRow {
    let operation = operation_name(input.operation);
    if !input.ec_spire_distributed_table {
        return unsupported(
            operation,
            "not_distributed_table",
            "not an ec_spire distributed coordinator table",
            None,
        );
    }
    if !input.single_table || input.has_join {
        return unsupported_v1(
            operation,
            "unsupported_join_shape",
            "ec_spire_distributed: joined DML is not yet supported in v1",
        );
    }
    if input.has_subquery {
        return unsupported_v1(
            operation,
            "unsupported_subquery_shape",
            "ec_spire_distributed: subquery DML is not yet supported in v1",
        );
    }
    if input.has_returning {
        return unsupported_v1(
            operation,
            "unsupported_returning_shape",
            "ec_spire_distributed: RETURNING is not yet supported in v1",
        );
    }
    if input.pk_column.is_empty()
        || input.predicate_column != Some(input.pk_column)
        || input.predicate_operator != Some("=")
        || !matches!(
            input.predicate_value_kind,
            SpireDmlFrontdoorValueKind::ConstBigint | SpireDmlFrontdoorValueKind::ParamBigint
        )
    {
        return unsupported_v1(
            operation,
            "unsupported_pk_predicate",
            "ec_spire_distributed: DML requires a bigint primary-key equality predicate in v1",
        );
    }

    match input.operation {
        SpireDmlFrontdoorOperation::Update => classify_update(input),
        SpireDmlFrontdoorOperation::Delete => supported(operation, "delete_by_pk"),
        SpireDmlFrontdoorOperation::PkSelect => classify_pk_select(input),
    }
}

fn classify_update(input: SpireDmlFrontdoorShapeInput<'_>) -> SpireDmlFrontdoorShapeRow {
    let operation = operation_name(input.operation);
    if input.updated_columns.is_empty() {
        return unsupported_v1(
            operation,
            "unsupported_empty_update",
            "ec_spire_distributed: UPDATE requires at least one target column in v1",
        );
    }
    if input
        .updated_columns
        .iter()
        .any(|column| *column == input.pk_column)
    {
        return unsupported_v1(
            operation,
            "unsupported_pk_update",
            "ec_spire_distributed: UPDATE of the primary-key column is not supported in v1",
        );
    }
    if input.updated_columns.iter().any(|column| {
        input
            .embedding_columns
            .iter()
            .any(|embedding_column| embedding_column == column)
    }) {
        return unsupported(
            operation,
            "embedding_update_rejected",
            "ec_spire_distributed: UPDATE of indexed embedding column is not supported on a distributed ec_spire table. Use DELETE + INSERT.",
            Some("Cross-shard atomic moves will be available in a future release."),
        );
    }
    supported(operation, "update_non_embedding_by_pk")
}

fn classify_pk_select(input: SpireDmlFrontdoorShapeInput<'_>) -> SpireDmlFrontdoorShapeRow {
    let operation = operation_name(input.operation);
    if input.projected_columns.is_empty() {
        return unsupported_v1(
            operation,
            "unsupported_empty_projection",
            "ec_spire_distributed: PK SELECT requires at least one projected column in v1",
        );
    }
    supported(operation, "pk_select_by_pk")
}

fn operation_name(operation: SpireDmlFrontdoorOperation) -> &'static str {
    match operation {
        SpireDmlFrontdoorOperation::Update => "update_non_embedding",
        SpireDmlFrontdoorOperation::Delete => "delete",
        SpireDmlFrontdoorOperation::PkSelect => "pk_select",
    }
}

fn supported(operation: &'static str, kind: &'static str) -> SpireDmlFrontdoorShapeRow {
    SpireDmlFrontdoorShapeRow {
        supported: true,
        operation,
        kind,
        status: "supported_v1_shape",
        error: None,
        hint: None,
    }
}

fn unsupported_v1(
    operation: &'static str,
    kind: &'static str,
    error: &'static str,
) -> SpireDmlFrontdoorShapeRow {
    unsupported(operation, kind, error, Some(ADR_069_HINT))
}

fn unsupported(
    operation: &'static str,
    kind: &'static str,
    error: &'static str,
    hint: Option<&'static str>,
) -> SpireDmlFrontdoorShapeRow {
    SpireDmlFrontdoorShapeRow {
        supported: false,
        operation,
        kind,
        status: "unsupported_shape",
        error: Some(error),
        hint,
    }
}

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
            "unsupported_pk_predicate"
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
}
