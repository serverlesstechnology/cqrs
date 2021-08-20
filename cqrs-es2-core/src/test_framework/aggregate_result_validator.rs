use crate::{
    errors::Error,
    events::IEvent,
};

/// Validation object for the `TestFramework` package.
pub struct AggregateResultValidator<E: IEvent> {
    result: Result<Vec<E>, Error>,
}

impl<E: IEvent> AggregateResultValidator<E> {
    pub fn new(result: Result<Vec<E>, Error>) -> Self {
        Self { result }
    }

    /// Verifies that the expected events have been produced by the
    /// command.
    pub fn then_expect_events(
        self,
        expected_events: Vec<E>,
    ) {
        let events = match self.result {
            Ok(x) => x,
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

    /// Verifies that an `Error` with the expected message is
    /// produced with the command.
    pub fn then_expect_error(
        self,
        error_message: &str,
    ) {
        let err = match self.result {
            Ok(events) => {
                panic!(
                    "expected error, received events: '{:?}'",
                    events
                );
            },
            Err(e) => e,
        };

        match err {
            Error::TechnicalError(err) => {
                panic!(
                    "expected user error but found technical error: \
                     {}",
                    err
                )
            },
            Error::UserError(err) => {
                assert_eq!(
                    err.message,
                    Some(error_message.to_string())
                );
            },
        }
    }
}
