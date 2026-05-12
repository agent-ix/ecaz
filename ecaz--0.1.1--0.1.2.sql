-- Remove the superseded SPIRE coordinator-side row materialization catalog.
-- The CustomScan read path returns remote tuple payloads directly and the
-- coordinator write path now tracks routing through ec_spire_placement.

DROP TABLE IF EXISTS ec_spire_remote_row_materialization;
