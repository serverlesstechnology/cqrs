use super::i_aggregate::IAggregate;

/// Returns the aggregate and context around it that is needed when
/// committing events
pub trait IAggregateContext<A: IAggregate> {
    /// The aggregate instance with all state loaded.
    fn aggregate(&self) -> &A;
}
