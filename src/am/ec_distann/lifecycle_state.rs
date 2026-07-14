//! Typed authorities for the four persisted DistANN lifecycle state machines.
//!
//! Catalog CHECK constraints protect stored spelling. These enums protect the
//! Rust side from distributing transition legality across endpoint-specific
//! string comparisons and UPDATE predicates.

use std::fmt;

pub(crate) trait LifecycleState: Copy + Eq + fmt::Display {
    fn as_str(self) -> &'static str;
    fn allows(self, next: Self) -> bool;
}

macro_rules! lifecycle_state {
    (
        $name:ident { $($variant:ident),+ $(,)? }
        transitions { $( $from:ident => [$($to:ident),* $(,)?] ),+ $(,)? }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub(crate) enum $name {
            $($variant),+
        }

        impl $name {
            pub(crate) fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($variant),)+
                }
            }

            pub(crate) fn parse(value: &str) -> Result<Self, String> {
                match value {
                    $(stringify!($variant) => Ok(Self::$variant),)+
                    other => Err(format!(
                        "EC_EPOCH_STATE: unknown {} state {other:?}",
                        stringify!($name)
                    )),
                }
            }
        }

        impl LifecycleState for $name {
            fn as_str(self) -> &'static str {
                self.as_str()
            }

            fn allows(self, next: Self) -> bool {
                match self {
                    $(Self::$from => [$(Self::$to),*].contains(&next),)+
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }
    };
}

lifecycle_state! {
    RegistrationState {
        Registered, Building, Ready, Aborted, Decided, Published, Cancelled
    }
    transitions {
        Registered => [Building, Ready, Aborted],
        Building => [Ready, Aborted],
        Ready => [Decided, Aborted],
        Aborted => [],
        Decided => [Published, Cancelled],
        Published => [],
        Cancelled => []
    }
}

lifecycle_state! {
    PublishDecisionState { Pending, Activated, Applied, Cancelled }
    transitions {
        Pending => [Activated, Applied, Cancelled],
        Activated => [Applied],
        Applied => [],
        Cancelled => []
    }
}

lifecycle_state! {
    GenerationState { Building, Ready, Published, Retired }
    transitions {
        Building => [Ready],
        Ready => [Published],
        Published => [Retired],
        Retired => []
    }
}

lifecycle_state! {
    PredecessorDisposition { Pending, Retired, Abandoned }
    transitions {
        Pending => [Retired, Abandoned],
        Retired => [],
        Abandoned => []
    }
}

pub(crate) fn require_transition<S: LifecycleState>(
    from: S,
    to: S,
    context: &str,
) -> Result<(), String> {
    if !from.allows(to) {
        return Err(format!(
            "EC_EPOCH_STATE: illegal {context} transition {from} -> {to}"
        ));
    }
    Ok(())
}

pub(crate) fn require_exact_transition<S: LifecycleState>(
    from: S,
    to: S,
    changed_rows: usize,
    context: &str,
) -> Result<(), String> {
    require_transition(from, to, context)?;
    if changed_rows != 1 {
        return Err(format!(
            "EC_EPOCH_STATE: {context} transition {from} -> {to} changed {changed_rows} rows, expected 1"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_transition_authority_rejects_terminal_and_skipped_edges() {
        assert!(RegistrationState::Registered.allows(RegistrationState::Ready));
        assert!(!RegistrationState::Ready.allows(RegistrationState::Published));
        assert!(!RegistrationState::Published.allows(RegistrationState::Ready));
        assert!(PublishDecisionState::Pending.allows(PublishDecisionState::Applied));
        assert!(!PublishDecisionState::Cancelled.allows(PublishDecisionState::Applied));
        assert!(GenerationState::Published.allows(GenerationState::Retired));
        assert!(!GenerationState::Building.allows(GenerationState::Published));
        assert!(PredecessorDisposition::Pending.allows(PredecessorDisposition::Abandoned));
        assert!(!PredecessorDisposition::Retired.allows(PredecessorDisposition::Pending));
    }

    #[test]
    fn exact_transition_requires_one_changed_row() {
        assert!(require_exact_transition(
            GenerationState::Building,
            GenerationState::Ready,
            1,
            "generation"
        )
        .is_ok());
        assert!(require_exact_transition(
            GenerationState::Building,
            GenerationState::Ready,
            0,
            "generation"
        )
        .is_err());
    }
}
