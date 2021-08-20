//! # events
//!
//! A central location for event interfaces

pub use event_context::EventContext;
pub use i_event::IEvent;
pub use i_event_consumer::IEventConsumer;
pub use i_event_handler::IEventHandler;

mod event_context;
mod i_event;
mod i_event_consumer;
mod i_event_handler;
