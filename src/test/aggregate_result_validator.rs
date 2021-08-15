use crate::{
    aggregates::IAggregate,
    errors::AggregateError,
};

/// Validation object for the `TestFramework` package.
pub struct AggregateResultValidator<A: IAggregate> {
    pub result: Result<Vec<A::Event>, AggregateError>,
}

impl<A: IAggregate> AggregateResultValidator<A> {
    /// Verifies that the expected events have been produced by the
    /// command.
    pub fn then_expect_events(
        self,
        expected_events: Vec<A::Event>,
    ) {
        let events = match self.result {
            Ok(expected_events) => expected_events,
            Err(err) => {
                panic!(
                    "expected success, received aggregate error: \
                     '{}'",
                    err
                );
            },
        };
        assert_eq!(&events[..], &expected_events[..]);
    }

    /// Verifies that an `AggregateError` with the expected message is
    /// produced with the command.
    pub fn then_expect_error(
        self,
        error_message: &str,
    ) {
        match self.result {
            Ok(events) => {
                panic!(
                    "expected error, received events: '{:?}'",
                    events
                );
            },
            Err(err) => {
                match err {
                    AggregateError::TechnicalError(err) => {
                        panic!(
                            "expected user error but found \
                             technical error: {}",
                            err
                        )
                    },
                    AggregateError::UserError(err) => {
                        assert_eq!(
                            err.message,
                            Some(error_message.to_string())
                        );
                    },
                }
            },
        };
    }
}
