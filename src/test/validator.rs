use crate::aggregate::Aggregate;

/// Validation object for the `TestFramework` package.
pub struct AggregateResultValidator<A>
where
    A: Aggregate,
{
    result: Result<Vec<A::Event>, A::Error>,
}

impl<A: Aggregate> AggregateResultValidator<A> {
    /// Verifies that the expected events have been produced by the command.
    ///
    /// ```
    /// # use cqrs_es::doc::{MyAggregate, MyCommands, MyEvents, MyService};
    /// # async fn test() {
    /// use cqrs_es::test::TestFramework;
    ///
    /// let validator = TestFramework::<MyAggregate>::with(MyService)
    ///     .given_no_previous_events()
    ///     .when(MyCommands::DoSomething);
    ///
    /// validator.then_expect_events(vec![MyEvents::SomethingWasDone]);
    /// # }
    /// ```
    pub fn then_expect_events(self, expected_events: Vec<A::Event>) {
        let events = self.result.unwrap_or_else(|err| {
            panic!("expected success, received aggregate error: '{err}'");
        });
        assert_eq!(events, expected_events);
    }

    /// Verifies that the result is a `UserError` and returns the internal error payload for
    /// further validation.
    ///
    /// ```
    /// # use cqrs_es::doc::{MyAggregate, MyCommands, MyEvents, MyService};
    /// use cqrs_es::test::TestFramework;
    ///
    /// let validator = TestFramework::<MyAggregate>::with(MyService)
    ///     .given_no_previous_events()
    ///     .when(MyCommands::BadCommand);
    ///
    /// validator.then_expect_error_message("the expected error message");
    /// ```
    pub fn then_expect_error_message(self, error_message: &str) {
        match self.result {
            Ok(events) => {
                panic!("expected error, received events: '{events:?}'");
            }
            Err(err) => assert_eq!(err.to_string(), error_message.to_string()),
        }
    }

    /// Returns the internal error payload for validation by the user.
    ///
    /// ```
    /// # use cqrs_es::doc::{MyAggregate, MyCommands, MyEvents, MyService, MyUserError};
    /// use cqrs_es::test::TestFramework;
    ///
    /// let validator = TestFramework::<MyAggregate>::with(MyService)
    ///     .given_no_previous_events()
    ///     .when(MyCommands::BadCommand);
    ///
    /// let expected = MyUserError("the expected error message".to_string());
    /// assert_eq!(expected,validator.inspect_result().unwrap_err());
    /// ```
    pub fn inspect_result(self) -> Result<Vec<A::Event>, A::Error> {
        self.result
    }

    pub(crate) fn new(result: Result<Vec<A::Event>, A::Error>) -> Self {
        Self { result }
    }
}
impl<A> AggregateResultValidator<A>
where
    A: Aggregate,
    A::Error: PartialEq,
{
    /// Verifies that the result is the expected error.
    ///
    /// > Note that the configured Error *must* implement `std::cmp::PartialEq`.
    ///
    /// ```
    /// # use cqrs_es::doc::{MyAggregate, MyCommands, MyEvents, MyService, MyUserError};
    /// use cqrs_es::test::TestFramework;
    ///
    /// let validator = TestFramework::<MyAggregate>::with(MyService)
    ///     .given_no_previous_events()
    ///     .when(MyCommands::BadCommand);
    ///
    /// let expected = MyUserError("the expected error message".to_string());
    /// validator.then_expect_error(expected);
    /// ```
    pub fn then_expect_error(self, expected_error: A::Error) {
        match self.result {
            Ok(events) => {
                panic!("expected error, received events: '{events:?}'");
            }
            Err(err) => {
                assert_eq!(err, expected_error);
            }
        }
    }
}
