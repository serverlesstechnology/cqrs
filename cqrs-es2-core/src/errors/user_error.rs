use std::{
    collections::HashMap,
    fmt::{
        Debug,
        Display,
        Formatter,
        Result as fmtResult,
    },
};

use serde::{
    Deserialize,
    Serialize,
};

/// Payload for an `Error::UserError`, somewhat modeled on
/// the errors produced by the [`validator`](https://github.com/Keats/validator) package. This payload implements `Serialize`
/// with the intention of allowing the user to return this object as
/// the response payload.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UserError {
    /// An optional code to indicate the a user-defined error.
    pub code: Option<String>,

    /// An optional message describing the error, meant to be
    /// returned to the user.
    pub message: Option<String>,

    /// Optional additional parameters for adding additional context
    /// to the error.
    pub params: Option<HashMap<String, String>>,
}

impl Display for UserError {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> fmtResult {
        write!(
            f,
            "UserError - code: {:?}\n  message: {:?}\n params: {:?}",
            &self.code, &self.message, &self.params
        )
    }
}
