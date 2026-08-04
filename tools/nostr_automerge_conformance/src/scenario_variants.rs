#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EvidenceRole {
    Control,
    LowerControl,
    Change,
    Dependency,
    InvalidCarrier,
    ValidCarrier,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScenarioEvidence<T> {
    pub(crate) id: T,
    pub(crate) role: EvidenceRole,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScenarioVariant<T> {
    pub(crate) name: &'static str,
    pub(crate) delivery: Vec<ScenarioEvidence<T>>,
}

pub(crate) fn adversarial_variants<T: Clone>(
    canonical: &[ScenarioEvidence<T>],
) -> Vec<ScenarioVariant<T>> {
    let mut duplicate_heavy = Vec::with_capacity(canonical.len().saturating_mul(3));
    for evidence in canonical {
        duplicate_heavy.extend([evidence.clone(), evidence.clone(), evidence.clone()]);
    }
    vec![
        ScenarioVariant {
            name: "duplicate_heavy",
            delivery: duplicate_heavy,
        },
        ScenarioVariant {
            name: "dependency_last",
            delivery: delay(canonical, |role| role == EvidenceRole::Dependency),
        },
        ScenarioVariant {
            name: "control_last",
            delivery: delay(canonical, |role| {
                matches!(role, EvidenceRole::Control | EvidenceRole::LowerControl)
            }),
        },
        ScenarioVariant {
            name: "invalid_before_valid_carrier",
            delivery: prioritize(canonical, |role| role == EvidenceRole::InvalidCarrier),
        },
        ScenarioVariant {
            name: "late_lower_control",
            delivery: delay(canonical, |role| role == EvidenceRole::LowerControl),
        },
    ]
}

fn delay<T: Clone>(
    canonical: &[ScenarioEvidence<T>],
    delayed: impl Fn(EvidenceRole) -> bool,
) -> Vec<ScenarioEvidence<T>> {
    canonical
        .iter()
        .filter(|item| !delayed(item.role))
        .cloned()
        .chain(canonical.iter().filter(|item| delayed(item.role)).cloned())
        .collect()
}

fn prioritize<T: Clone>(
    canonical: &[ScenarioEvidence<T>],
    first: impl Fn(EvidenceRole) -> bool,
) -> Vec<ScenarioEvidence<T>> {
    canonical
        .iter()
        .filter(|item| first(item.role))
        .cloned()
        .chain(canonical.iter().filter(|item| !first(item.role)).cloned())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{EvidenceRole, ScenarioEvidence, adversarial_variants};

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Completion {
        Complete,
        Pending,
    }

    fn completion(delivery: &[ScenarioEvidence<u8>]) -> Completion {
        let roles = delivery.iter().map(|item| item.role).collect::<Vec<_>>();
        if roles.contains(&EvidenceRole::Control) && roles.contains(&EvidenceRole::Dependency) {
            Completion::Complete
        } else {
            Completion::Pending
        }
    }

    fn canonical_report(delivery: &[ScenarioEvidence<u8>]) -> BTreeSet<u8> {
        delivery
            .iter()
            .filter(|item| item.role != EvidenceRole::InvalidCarrier)
            .map(|item| item.id)
            .collect()
    }

    #[test]
    fn add_duplicate_and_delayed_evidence_scenario_families() {
        let canonical = vec![
            ScenarioEvidence {
                id: 1,
                role: EvidenceRole::LowerControl,
            },
            ScenarioEvidence {
                id: 2,
                role: EvidenceRole::Control,
            },
            ScenarioEvidence {
                id: 3,
                role: EvidenceRole::Dependency,
            },
            ScenarioEvidence {
                id: 4,
                role: EvidenceRole::Change,
            },
            ScenarioEvidence {
                id: 5,
                role: EvidenceRole::InvalidCarrier,
            },
            ScenarioEvidence {
                id: 5,
                role: EvidenceRole::ValidCarrier,
            },
        ];
        let variants = adversarial_variants(&canonical);
        assert_eq!(
            variants
                .iter()
                .map(|variant| variant.name)
                .collect::<Vec<_>>(),
            vec![
                "duplicate_heavy",
                "dependency_last",
                "control_last",
                "invalid_before_valid_carrier",
                "late_lower_control",
            ]
        );
        assert!(
            variants
                .iter()
                .all(|variant| canonical_report(&variant.delivery) == canonical_report(&canonical))
        );
        let dependency_last = &variants[1].delivery;
        assert_eq!(
            completion(&dependency_last[..dependency_last.len() - 1]),
            Completion::Pending
        );
        assert_eq!(completion(dependency_last), Completion::Complete);
        let control_last = &variants[2].delivery;
        let first_control = control_last
            .iter()
            .position(|item| item.role == EvidenceRole::Control);
        assert!(first_control.is_some());
        let Some(first_control) = first_control else {
            return;
        };
        assert_eq!(
            completion(&control_last[..first_control]),
            Completion::Pending
        );
        assert_eq!(completion(control_last), Completion::Complete);
    }
}
