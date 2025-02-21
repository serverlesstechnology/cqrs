//! This module provides a test framework for building a resilient test base around aggregates.
//!
//! A `TestFramework` should be used to build a comprehensive set of aggregate tests to verify
//! your application logic.
//!
//! ```rust
//! # use cqrs_es::test::TestFramework;
//! # use cqrs_es::doc::{Customer, CustomerEvent, CustomerCommand, CustomerService};
//! # fn test() {
//! type CustomerTestFramework = TestFramework<Customer>;
//!
//! CustomerTestFramework::with(CustomerService::default())
//!     .given_no_previous_events()
//!     .when(CustomerCommand::AddCustomerName{
//!             name: "John Doe".to_string()
//!         })
//!     .then_expect_events(vec![
//!         CustomerEvent::NameAdded{
//!             name: "John Doe".to_string()
//!         }]);
//! # }
//! ```
mod executor;
mod framework;
mod validator;

pub use crate::test::executor::*;
pub use crate::test::framework::*;
pub use crate::test::validator::*;
