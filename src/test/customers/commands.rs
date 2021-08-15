use std::fmt::Debug;

use crate::IDomainCommand;

#[derive(Debug, PartialEq)]
pub enum CustomerCommand {
    AddCustomerName(AddCustomerName),
    UpdateEmail(UpdateEmail),
}

#[derive(Debug, PartialEq)]
pub struct AddCustomerName {
    pub changed_name: String,
}

#[derive(Debug, PartialEq)]
pub struct UpdateEmail {
    pub new_email: String,
}

impl IDomainCommand for CustomerCommand {}
