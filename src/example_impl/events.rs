use serde::{
    Deserialize,
    Serialize,
};
use std::fmt::Debug;

use crate::IDomainEvent;

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub enum CustomerEvent {
    NameAdded(NameAdded),
    EmailUpdated(EmailUpdated),
    AddressUpdated(AddressUpdated),
}

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub struct NameAdded {
    pub changed_name: String,
}

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub struct EmailUpdated {
    pub new_email: String,
}

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub struct AddressUpdated {
    pub new_address: String,
}

impl IDomainEvent for CustomerEvent {}
