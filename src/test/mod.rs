pub use test_framework::TestFramework;

mod aggregate_result_validator;
mod aggregate_test_executor;

mod test_framework;

#[doc(hidden)]
pub mod customer;

#[cfg(test)]
mod test_customer;
