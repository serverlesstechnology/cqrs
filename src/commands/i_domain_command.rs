use std::fmt::Debug;

/// A `DomainCommand` represents business API call.
///
/// The name of a `DomainCommand` should always be in the present
/// tense, e.g.,
/// - `ChangeEmailAddress`
/// - `AddDependency`
///
/// To simplify serialization, a command should be an enum, and each
/// element should have a payload. By convention, the payload has the
/// same name as the element, and elements that do not require
/// additional information use an empty payload.
///
/// Though the `DomainCommand` trait only has a single function, the
/// commands must also derive a number of standard traits.
/// - `Debug` and `PartialEq` - needed for effective testing
///
/// # Examples
/// ```rust
/// use cqrs_es2::IDomainCommand;
/// use std::fmt::Debug;
///
/// #[derive(Debug, PartialEq)]
/// pub enum CustomerCommand {
///     ChangeName(ChangeName),
///     UpdateEmail(UpdateEmail),
/// }
///
/// #[derive(Debug, PartialEq)]
/// pub struct ChangeName {
///     new_name: String,
/// }
///
/// #[derive(Debug, PartialEq)]
/// pub struct UpdateEmail {
///     new_email: String,
/// }
///
/// impl IDomainCommand for CustomerCommand {};
/// ```
pub trait IDomainCommand: Debug + PartialEq + Sync + Send {}
