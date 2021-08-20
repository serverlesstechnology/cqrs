use std::fmt::Debug;

/// An `ICommand` represents business API call.
///
/// The name of an `ICommand` should always be in the present
/// tense, e.g.,
/// - `ChangeEmailAddress`
/// - `AddDependency`
///
/// To simplify serialization, a command should be an enum, and each
/// element should have a payload. By convention, the payload has the
/// same name as the element, and elements that do not require
/// additional information use an empty payload.
///
/// Though the `ICommand` trait only has a single function, the
/// commands must also derive a number of standard traits.
/// - `Clone` - events may be cloned throughout the framework
/// - `Debug` and `PartialEq` - needed for effective testing
///
/// # Examples
/// ```rust
/// use std::fmt::Debug;
///
/// use cqrs_es2_core::ICommand;
///
/// #[derive(Debug, PartialEq, Clone)]
/// pub enum CustomerCommand {
///     AddCustomerName(AddCustomerName),
///     UpdateEmail(UpdateEmail),
/// }
///
/// #[derive(Debug, PartialEq, Clone)]
/// pub struct AddCustomerName {
///     changed_name: String,
/// }
///
/// #[derive(Debug, PartialEq, Clone)]
/// pub struct UpdateEmail {
///     new_email: String,
/// }
///
/// impl ICommand for CustomerCommand {};
/// ```
pub trait ICommand: Debug + PartialEq + Clone + Sync + Send {}
