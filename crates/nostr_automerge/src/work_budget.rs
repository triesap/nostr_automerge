use core::fmt;

/// A deterministic unit of evaluator work.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum WorkCounter {
    /// One retained raw-event observation inspected by evaluation.
    Event,
    /// One kind-specific carrier classification or validation operation.
    Carrier,
    /// One control candidate or transition operation.
    Control,
    /// One dependency-graph node operation.
    GraphNode,
    /// One dependency-graph edge operation.
    GraphEdge,
    /// One raw or decoded Automerge byte inspected.
    DecodeByte,
    /// One Automerge change application or materialization operation.
    ApplyChange,
    /// One checkpoint carrier, snapshot, or proof byte inspected.
    CheckpointByte,
    /// One checkpoint carrier, chunk, proof, graph, history, or projection item inspected.
    CheckpointItem,
    /// One typed state assertion or projected value operation.
    Assertion,
}

impl WorkCounter {
    /// Returns the stable machine-readable counter name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Event => "event",
            Self::Carrier => "carrier",
            Self::Control => "control",
            Self::GraphNode => "graph_node",
            Self::GraphEdge => "graph_edge",
            Self::DecodeByte => "decode_byte",
            Self::ApplyChange => "apply_change",
            Self::CheckpointByte => "checkpoint_byte",
            Self::CheckpointItem => "checkpoint_item",
            Self::Assertion => "assertion",
        }
    }
}

/// Exact deterministic work consumed by an evaluation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorkCounters {
    event: u64,
    carrier: u64,
    control: u64,
    graph_node: u64,
    graph_edge: u64,
    decode_byte: u64,
    apply_change: u64,
    checkpoint_byte: u64,
    checkpoint_item: u64,
    assertion: u64,
}

impl WorkCounters {
    /// Returns the consumed amount for one deterministic work dimension.
    #[must_use]
    pub const fn get(self, counter: WorkCounter) -> u64 {
        match counter {
            WorkCounter::Event => self.event,
            WorkCounter::Carrier => self.carrier,
            WorkCounter::Control => self.control,
            WorkCounter::GraphNode => self.graph_node,
            WorkCounter::GraphEdge => self.graph_edge,
            WorkCounter::DecodeByte => self.decode_byte,
            WorkCounter::ApplyChange => self.apply_change,
            WorkCounter::CheckpointByte => self.checkpoint_byte,
            WorkCounter::CheckpointItem => self.checkpoint_item,
            WorkCounter::Assertion => self.assertion,
        }
    }

    fn set(&mut self, counter: WorkCounter, value: u64) {
        match counter {
            WorkCounter::Event => self.event = value,
            WorkCounter::Carrier => self.carrier = value,
            WorkCounter::Control => self.control = value,
            WorkCounter::GraphNode => self.graph_node = value,
            WorkCounter::GraphEdge => self.graph_edge = value,
            WorkCounter::DecodeByte => self.decode_byte = value,
            WorkCounter::ApplyChange => self.apply_change = value,
            WorkCounter::CheckpointByte => self.checkpoint_byte = value,
            WorkCounter::CheckpointItem => self.checkpoint_item = value,
            WorkCounter::Assertion => self.assertion = value,
        }
    }
}

/// Deterministic local capacity counters, separate from protocol validity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkBudget {
    remaining_bytes: u64,
    remaining_items: u64,
    consumed: WorkCounters,
}

impl WorkBudget {
    /// Creates a caller-selected local budget.
    #[must_use]
    pub const fn new(max_bytes: u64, max_items: u64) -> Self {
        Self {
            remaining_bytes: max_bytes,
            remaining_items: max_items,
            consumed: WorkCounters {
                event: 0,
                carrier: 0,
                control: 0,
                graph_node: 0,
                graph_edge: 0,
                decode_byte: 0,
                apply_change: 0,
                checkpoint_byte: 0,
                checkpoint_item: 0,
                assertion: 0,
            },
        }
    }

    /// Charges one typed deterministic work dimension atomically.
    ///
    /// Decode and checkpoint bytes consume the byte ceiling. Every other
    /// counter consumes the item ceiling. Exhaustion or arithmetic overflow
    /// leaves both the remaining capacity and consumed counters unchanged.
    pub fn charge(&mut self, counter: WorkCounter, amount: u64) -> Result<(), BudgetExhausted> {
        let next_count = self
            .consumed
            .get(counter)
            .checked_add(amount)
            .ok_or(BudgetExhausted { counter })?;
        let bytes = matches!(
            counter,
            WorkCounter::DecodeByte | WorkCounter::CheckpointByte
        );
        let remaining = if bytes {
            self.remaining_bytes
        } else {
            self.remaining_items
        };
        let next_remaining = remaining
            .checked_sub(amount)
            .ok_or(BudgetExhausted { counter })?;
        if bytes {
            self.remaining_bytes = next_remaining;
        } else {
            self.remaining_items = next_remaining;
        }
        self.consumed.set(counter, next_count);
        Ok(())
    }

    /// Charges deterministic byte work, failing without changing the budget.
    pub fn charge_bytes(&mut self, amount: u64) -> Result<(), BudgetExhausted> {
        self.charge(WorkCounter::DecodeByte, amount)
    }

    /// Charges deterministic item work, failing without changing the budget.
    pub fn charge_items(&mut self, amount: u64) -> Result<(), BudgetExhausted> {
        self.charge(WorkCounter::GraphNode, amount)
    }

    pub(crate) fn charge_checkpoint_bytes(&mut self, amount: u64) -> Result<(), BudgetExhausted> {
        self.charge(WorkCounter::CheckpointByte, amount)
    }

    pub(crate) fn charge_checkpoint_items(&mut self, amount: u64) -> Result<(), BudgetExhausted> {
        self.charge(WorkCounter::CheckpointItem, amount)
    }

    /// Returns the remaining byte and item counters.
    #[must_use]
    pub const fn remaining(self) -> (u64, u64) {
        (self.remaining_bytes, self.remaining_items)
    }

    /// Returns exact successfully consumed deterministic work.
    #[must_use]
    pub const fn consumed(self) -> WorkCounters {
        self.consumed
    }

    #[cfg(test)]
    pub(crate) const fn unlimited_for_test() -> Self {
        Self::new(u64::MAX, u64::MAX)
    }
}

/// A local deterministic work counter was exhausted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BudgetExhausted {
    counter: WorkCounter,
}

impl BudgetExhausted {
    /// Returns the work dimension whose charge could not be completed.
    #[must_use]
    pub const fn counter(self) -> WorkCounter {
        self.counter
    }
}

impl fmt::Display for BudgetExhausted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "local {} work budget exhausted",
            self.counter.as_str()
        )
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
    use super::{CancellationCheck, NeverCancelled, WorkBudget, WorkCounter, WorkCounters};

    #[test]
    fn deterministic_work_dimensions_are_independent() {
        let counters = WorkCounters::default();
        for counter in [
            WorkCounter::Event,
            WorkCounter::Carrier,
            WorkCounter::Control,
            WorkCounter::GraphNode,
            WorkCounter::GraphEdge,
            WorkCounter::DecodeByte,
            WorkCounter::ApplyChange,
            WorkCounter::CheckpointByte,
            WorkCounter::CheckpointItem,
            WorkCounter::Assertion,
        ] {
            assert_eq!(counters.get(counter), 0);
        }
    }

    #[test]
    fn failed_charge_does_not_mutate_budget() {
        let mut budget = WorkBudget::new(4, 2);
        let exhausted = budget.charge(WorkCounter::CheckpointByte, 5);
        assert!(matches!(
            exhausted,
            Err(error) if error.counter() == WorkCounter::CheckpointByte
        ));
        assert_eq!(budget.remaining(), (4, 2));
        assert_eq!(budget.consumed(), WorkCounters::default());
        assert!(budget.charge(WorkCounter::Control, 2).is_ok());
        assert_eq!(budget.remaining(), (4, 0));
        assert_eq!(budget.consumed().get(WorkCounter::Control), 2);
        assert_eq!(budget.consumed().get(WorkCounter::GraphNode), 0);
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
