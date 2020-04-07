use std::{error, fmt};
use std::hash::Hash;

/// A trait that defines an `Aggregate`, the fundamental component in CQRS that encapsulates the
/// state and business logic for the application. An `Aggregate` is always an entity along with
/// all objects associated with it.
///
/// In [DDD](https://en.wikipedia.org/wiki/Domain-driven_design) we require that changes are made
/// only after loading the full `Aggregate` in order to ensure that the full context is understood.
///
/// #Example
/// ```rust
/// # use cqrs_es::aggregate::Aggregate;
/// struct Customer {
///     customer_id: String,
///     name: String,
///     email: String,
/// }
///
/// impl Aggregate for Customer {
///     fn aggregate_type() -> &'static str { "customer" }
/// }
///
/// impl Default for Customer {fn default() -> Self {
///         Customer {
///             customer_id: "".to_string(),
///             name: "".to_string(),
///             email: "".to_string(),
///         }
///     }
/// }
/// ```
pub trait Aggregate: Default {
    /// aggregate_type is a unique identifier for this aggregate
    fn aggregate_type() -> &'static str;
}

/// An `AggregateId` specifies a unique instance of an
/// The [newtype pattern](https://doc.rust-lang.org/book/ch19-04-advanced-types.html#using-the-newtype-pattern-for-type-safety-and-abstraction)
/// is encouraged here and a future `derive` will support this.
pub trait AggregateId<A: Aggregate>: ToString + Eq + Hash {
    /// aggregate_type specifies the unique `Aggregate` that this ID refers to.
    /// This value should be identical to that of the [`Aggregate::aggregate_type()`]
    fn aggregate_type(&self) -> &'static str;
}

/// The base error for the framework.
#[derive(Debug, PartialEq)]
pub enum AggregateError {
    /// The user has made an error, a String value contains a message to be delivered to the user.
    UserError(String),
    /// A technical error was encountered that prevented the command from being applied to the
    /// aggregate. In general the accompanying message should be logged for investigation rather
    /// than returned to the user.
    TechnicalError(String),
}

impl error::Error for AggregateError {}

impl fmt::Display for AggregateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregateError::TechnicalError(message) => {
                write!(f, "{}", message)
            },
            AggregateError::UserError(message) => {
                write!(f, "{}", message)
            },
        }
    }
}

impl AggregateError {
    /// Convenience function to construct a [`UserError`] from a `&str`.
    pub fn new(msg: &str) -> Self { AggregateError::UserError(msg.to_string()) }
}

impl From<serde_json::error::Error> for AggregateError {
    fn from(err: serde_json::error::Error) -> Self {
        match err.classify() {
            serde_json::error::Category::Syntax => AggregateError::new("invalid json"),
            serde_json::error::Category::Io |
            serde_json::error::Category::Data |
            serde_json::error::Category::Eof => AggregateError::new("fail"),
        }
    }
}

// impl From<StorageError> for AggregateError {
//     fn from(e: StorageError) -> Self {
//         println!("error encountered storing events - {}", e.message);
//         AggregateError::new("server encountered an unknown error")
//     }
// }

