#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compaction_leaf_pid_match_rejects_malformed_header_pid() {
        let error = require_compaction_leaf_pid_match(42, 43).unwrap_err();

        assert_eq!(
            error,
            "ec_spire vacuum compaction leaf pid mismatch: manifest pid 42, object header pid 43"
        );
    }

    #[test]
    fn compaction_leaf_pid_match_returns_manifest_pid() {
        assert_eq!(require_compaction_leaf_pid_match(42, 42), Ok(42));
    }

    #[test]
    fn compaction_leaf_object_version_match_rejects_malformed_header_version() {
        let error = require_compaction_leaf_object_version_match(7, 8, 42).unwrap_err();

        assert_eq!(
            error,
            "ec_spire vacuum compaction leaf object_version mismatch for pid 42: manifest object_version 7, object header object_version 8"
        );
    }

    #[test]
    fn compaction_leaf_object_version_match_returns_manifest_version() {
        assert_eq!(
            require_compaction_leaf_object_version_match(7, 7, 42),
            Ok(7)
        );
    }
}
