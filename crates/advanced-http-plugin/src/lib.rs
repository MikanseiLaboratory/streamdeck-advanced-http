//! Stream Deck plugin that sends HTTP requests from keys and dials.
//!
//! The binary and the TypeScript contract generator both link this crate so the
//! `inventory` registrations for actions and settings stay in the build.

pub mod actions;
pub mod contracts;
pub mod request;
pub mod state;

/// Keep action registration linked into the binary and the type generator.
pub fn force_link() {
    actions::force_link();
}
