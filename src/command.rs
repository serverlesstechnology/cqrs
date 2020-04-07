use crate::aggregate::{Aggregate, AggregateError};
use crate::event::DomainEvent;

pub trait Command<A, E>
    where A: Aggregate,
          E: DomainEvent<A>
{
    fn handle(&self, aggregate: &mut A) -> Result<Vec<E>, AggregateError>;
}