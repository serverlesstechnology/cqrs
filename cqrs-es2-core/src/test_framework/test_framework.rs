use crate::{
    aggregates::IAggregate,
    commands::ICommand,
    events::IEvent,
};
use std::marker::PhantomData;

use super::aggregate_test_executor::AggregateTestExecutor;

/// A framework for rigorously testing the aggregate logic, one of the
/// ***most important*** parts of any CQRS system.
///
/// ```rust
/// use cqrs_es2_core::{
///     example_impl::{
///         AddCustomerName,
///         Customer,
///         CustomerCommand,
///         CustomerEvent,
///         NameAdded,
///     },
///     TestFramework,
/// };
///
/// type CustomerTestFramework =
///     TestFramework<CustomerCommand, CustomerEvent, Customer>;
///
/// CustomerTestFramework::default()
///     .given_no_previous_events()
///     .when(CustomerCommand::AddCustomerName(
///         AddCustomerName {
///             changed_name: "John Doe".to_string(),
///         },
///     ))
///     .then_expect_events(vec![CustomerEvent::NameAdded(
///         NameAdded {
///             changed_name: "John Doe".to_string(),
///         },
///     )]);
///
/// CustomerTestFramework::default()
///     .given(vec![CustomerEvent::NameAdded(
///         NameAdded {
///             changed_name: "John Doe".to_string(),
///         },
///     )])
///     .when(CustomerCommand::AddCustomerName(
///         AddCustomerName {
///             changed_name: "John Doe".to_string(),
///         },
///     ))
///     .then_expect_error(
///         "a name has already been added for this customer",
///     )
/// ```
pub struct TestFramework<C: ICommand, E: IEvent, A: IAggregate<C, E>>
{
    _phantom: PhantomData<(C, E, A)>,
}

impl<C: ICommand, E: IEvent, A: IAggregate<C, E>>
    TestFramework<C, E, A>
{
    /// Initiates an aggregate test with no previous events.
    #[must_use]
    pub fn given_no_previous_events(
        &self
    ) -> AggregateTestExecutor<C, E, A> {
        AggregateTestExecutor::new(Vec::new())
    }

    /// Initiates an aggregate test with a collection of previous
    /// events.
    #[must_use]
    pub fn given(
        &self,
        events: Vec<E>,
    ) -> AggregateTestExecutor<C, E, A> {
        AggregateTestExecutor::new(events)
    }
}

impl<C: ICommand, E: IEvent, A: IAggregate<C, E>> Default
    for TestFramework<C, E, A>
{
    fn default() -> Self {
        TestFramework {
            _phantom: PhantomData,
        }
    }
}
