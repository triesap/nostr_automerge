use crate::WorkCounter;

/// Returns whether any ordered control member satisfies `predicate`.
///
/// The caller owns the work policy. Every iterator pull, including the final
/// absent pull, and every predicate evaluation occurs only after its own
/// successful control-work observation.
pub(crate) fn any_control_member_metered<T, E>(
    members: &[T],
    mut predicate: impl FnMut(&T) -> bool,
    mut visit: impl FnMut(WorkCounter) -> Result<(), E>,
) -> Result<bool, E> {
    let mut members = members.iter();
    loop {
        visit(WorkCounter::Control)?;
        let Some(member) = members.next() else {
            return Ok(false);
        };
        visit(WorkCounter::Control)?;
        if predicate(member) {
            return Ok(true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::any_control_member_metered;
    use crate::{Completion, WorkBudget, WorkCounter};

    #[test]
    fn member_authorization_charges_each_pull_and_predicate_before_work() {
        let members = [10_u8, 20, 30, 40];
        for (needle, expected, expected_charges, expected_predicates) in [
            (5, false, 9, 4),
            (10, true, 2, 1),
            (20, true, 4, 2),
            (40, true, 8, 4),
        ] {
            let charges = std::cell::Cell::new(0_usize);
            let predicates = std::cell::Cell::new(0_usize);
            assert_eq!(
                any_control_member_metered(
                    &members,
                    |member| {
                        predicates.set(predicates.get().saturating_add(1));
                        *member == needle
                    },
                    |counter| {
                        assert_eq!(counter, WorkCounter::Control);
                        charges.set(charges.get().saturating_add(1));
                        Ok::<(), Completion>(())
                    },
                ),
                Ok(expected)
            );
            assert_eq!(charges.get(), expected_charges);
            assert_eq!(predicates.get(), expected_predicates);
        }

        let empty: [u8; 0] = [];
        let charges = std::cell::Cell::new(0_usize);
        let predicates = std::cell::Cell::new(0_usize);
        assert_eq!(
            any_control_member_metered(
                &empty,
                |_| {
                    predicates.set(predicates.get().saturating_add(1));
                    false
                },
                |_| {
                    charges.set(charges.get().saturating_add(1));
                    Ok::<(), Completion>(())
                },
            ),
            Ok(false)
        );
        assert_eq!(charges.get(), 1);
        assert_eq!(predicates.get(), 0);
    }

    #[test]
    fn member_authorization_preserves_every_budget_and_cancellation_boundary() {
        let members = [10_u8, 20, 30, 40];
        let required = members.len().saturating_mul(2).saturating_add(1);
        for capacity in 0..=required {
            let predicates = std::cell::Cell::new(0_usize);
            let mut budget = WorkBudget::new(0, u64::try_from(capacity).unwrap_or(u64::MAX));
            let result = any_control_member_metered(
                &members,
                |_| {
                    predicates.set(predicates.get().saturating_add(1));
                    false
                },
                |counter| {
                    budget
                        .charge(counter, 1)
                        .map_err(|_| Completion::BudgetExhausted)
                },
            );
            if capacity < required {
                assert_eq!(result, Err(Completion::BudgetExhausted));
            } else {
                assert_eq!(result, Ok(false));
            }
            assert_eq!(predicates.get(), capacity.min(required) / 2);
        }

        for cancel_at in 0..required {
            let charges = std::cell::Cell::new(0_usize);
            let predicates = std::cell::Cell::new(0_usize);
            let result = any_control_member_metered(
                &members,
                |_| {
                    predicates.set(predicates.get().saturating_add(1));
                    false
                },
                |_| {
                    if charges.get() == cancel_at {
                        return Err(Completion::Cancelled);
                    }
                    charges.set(charges.get().saturating_add(1));
                    Ok(())
                },
            );
            assert_eq!(result, Err(Completion::Cancelled));
            assert_eq!(charges.get(), cancel_at);
            assert_eq!(predicates.get(), cancel_at / 2);
        }
    }
}
