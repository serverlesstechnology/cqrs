use serde::{Deserialize, Serialize};

use crate::{Aggregate, AggregateError, Command, DomainEvent};

#[derive(Serialize, Deserialize)]
pub struct Customer {
    pub customer_id: String,
    pub name: String,
    pub email: String,
}

impl Aggregate for Customer {
    fn aggregate_type() -> &'static str { "customer" }
}

impl Default for Customer {
    fn default() -> Self {
        Customer {
            customer_id: "".to_string(),
            name: "".to_string(),
            email: "".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CustomerEvent {
    NameAdded(NameAdded),
    EmailUpdated(EmailUpdated),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NameAdded {
    pub changed_name: String
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EmailUpdated {
    pub new_email: String
}

impl DomainEvent<Customer> for CustomerEvent {
    fn apply(self, customer: &mut Customer) {
        match self {
            CustomerEvent::NameAdded(payload) => {
                customer.name = payload.changed_name;
            }
            CustomerEvent::EmailUpdated(payload) => {
                customer.email = payload.new_email;
            }
        }
    }
}

pub struct AddCustomerName {
    pub changed_name: String
}

pub struct UpdateEmail {
    pub new_email: String
}

impl Command<Customer, CustomerEvent> for AddCustomerName {
    fn handle(self, customer: &Customer) -> Result<Vec<CustomerEvent>, AggregateError> {
        if customer.name.as_str() != "" {
            return Err(AggregateError::new("a name has already been added for this customer"));
        }
        let payload = NameAdded {
            changed_name: self.changed_name
        };
        Ok(vec![CustomerEvent::NameAdded(payload)])
    }
}

#[cfg(test)]
mod doc_tests {
    use super:: *;
    use crate::test::TestFramework;

    type CustomerTestFramework = TestFramework<Customer, CustomerEvent>;

    #[test]
    fn test_add_name() {
        CustomerTestFramework::default()
            .given_no_previous_events()
            .when(AddCustomerName { changed_name: "John Doe".to_string() })
            .then_expect_events(vec![
                CustomerEvent::NameAdded(NameAdded {
                    changed_name: "John Doe".to_string()
                })
            ]);
    }

    #[test]
    fn test_add_name_again() {
        CustomerTestFramework::default()
            .given(vec![
                CustomerEvent::NameAdded(NameAdded {
                    changed_name: "John Doe".to_string()
                })
            ])
            .when(AddCustomerName { changed_name: "John Doe".to_string() })
            .then_expect_error("a name has already been added for this customer");
    }
}
