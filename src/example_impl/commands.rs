use std::fmt::Debug;

use crate::ICommand;

#[derive(Debug, PartialEq, Clone)]
pub enum CustomerCommand {
    AddCustomerName(AddCustomerName),
    UpdateEmail(UpdateEmail),
    AddAddress(AddAddress),
}

#[derive(Debug, PartialEq, Clone)]
pub struct AddCustomerName {
    pub changed_name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct UpdateEmail {
    pub new_email: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AddAddress {
    pub new_address: String,
}

impl ICommand for CustomerCommand {}
