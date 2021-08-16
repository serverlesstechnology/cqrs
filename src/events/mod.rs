//! # events
//!
//! A central location for event interfaces

pub use event_context::EventContext;
pub use i_domain_event::IEvent;

mod event_context;
mod i_domain_event;
