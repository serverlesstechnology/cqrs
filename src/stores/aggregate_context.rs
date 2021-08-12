use crate::aggregates::Aggregate;

/// Returns the aggregate and context around it that is needed when
/// committing events
pub trait AggregateContext<A>
where
    A: Aggregate, {
    /// The aggregate instance with all state loaded.
    fn aggregate(&self) -> &A;
}
