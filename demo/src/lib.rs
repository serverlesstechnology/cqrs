#![forbid(unsafe_code)]
#![deny(clippy::all)]

pub mod command_extractor;
mod config;
mod domain;
mod queries;
pub mod route_handler;
mod services;
pub mod state;
