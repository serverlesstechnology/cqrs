use std::fmt::Debug;

use crate::ICommand;

#[derive(Debug, PartialEq)]
pub enum CustomerCommand {
    AddCustomerName(AddCustomerName),
    UpdateEmail(UpdateEmail),
    AddAddress(AddAddress),
}

#[derive(Debug, PartialEq)]
pub struct AddCustomerName {
    pub changed_name: String,
}

#[derive(Debug, PartialEq)]
pub struct UpdateEmail {
    pub new_email: String,
}

#[derive(Debug, PartialEq)]
pub struct AddAddress {
    pub new_address: String,
}

impl ICommand for CustomerCommand {}
