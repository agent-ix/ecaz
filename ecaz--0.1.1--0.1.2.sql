-- Remove the superseded SPIRE coordinator-side row materialization catalog.
--
-- 0.1.1 introduced ec_spire_remote_row_materialization for the Shape-A
-- distributed-read design: remote-origin AM scan rows needed a coordinator
-- heap mirror row before the index AM could return a visible heap TID. The
-- 0.1.2 Shape-B CustomScan pivot returns remote tuple payloads directly
-- through EcSpireDistributedScan instead, and ADR-069 coordinator-routed
-- writes track ownership in ec_spire_placement. The old catalog is therefore
-- intentionally dropped during the upgrade.

DROP TABLE IF EXISTS ec_spire_remote_row_materialization;
