use core::fmt;

/// Deterministic local capacity counters, separate from protocol validity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkBudget {
    remaining_bytes: u64,
    remaining_items: u64,
}

impl WorkBudget {
    /// Creates a caller-selected local budget.
    #[must_use]
    pub const fn new(max_bytes: u64, max_items: u64) -> Self {
        Self {
            remaining_bytes: max_bytes,
            remaining_items: max_items,
        }
    }

    /// Charges deterministic byte work, failing without changing the budget.
    pub fn charge_bytes(&mut self, amount: u64) -> Result<(), BudgetExhausted> {
        let remaining = self
            .remaining_bytes
            .checked_sub(amount)
            .ok_or(BudgetExhausted)?;
        self.remaining_bytes = remaining;
        Ok(())
    }

    /// Charges deterministic item work, failing without changing the budget.
    pub fn charge_items(&mut self, amount: u64) -> Result<(), BudgetExhausted> {
        let remaining = self
            .remaining_items
            .checked_sub(amount)
            .ok_or(BudgetExhausted)?;
        self.remaining_items = remaining;
        Ok(())
    }

    /// Returns the remaining byte and item counters.
    #[must_use]
    pub const fn remaining(self) -> (u64, u64) {
        (self.remaining_bytes, self.remaining_items)
    }

    #[cfg(test)]
    pub(crate) const fn unlimited_for_test() -> Self {
        Self::new(u64::MAX, u64::MAX)
    }
}

/// A local deterministic work counter was exhausted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BudgetExhausted;

impl fmt::Display for BudgetExhausted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("local work budget exhausted")
    }
}

impl std::error::Error for BudgetExhausted {}

/// Cooperative cancellation checked only at deterministic algorithm boundaries.
pub trait CancellationCheck {
    /// Returns true when the caller requests local evaluation to stop.
    fn is_cancelled(&self) -> bool;
}

impl<F> CancellationCheck for F
where
    F: Fn() -> bool,
{
    fn is_cancelled(&self) -> bool {
        self()
    }
}

/// A cancellation policy that never stops evaluation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NeverCancelled;

impl CancellationCheck for NeverCancelled {
    fn is_cancelled(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{CancellationCheck, NeverCancelled, WorkBudget};

    #[test]
    fn failed_charge_does_not_mutate_budget() {
        let mut budget = WorkBudget::new(4, 2);
        assert!(budget.charge_bytes(5).is_err());
        assert_eq!(budget.remaining(), (4, 2));
        assert!(budget.charge_items(2).is_ok());
        assert_eq!(budget.remaining(), (4, 0));
    }

    #[test]
    fn cancellation_has_no_clock_dependency() {
        assert!(!NeverCancelled.is_cancelled());
        assert!((|| true).is_cancelled());
        assert_eq!(
            WorkBudget::unlimited_for_test().remaining(),
            (u64::MAX, u64::MAX)
        );
    }
}
