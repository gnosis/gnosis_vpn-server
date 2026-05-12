// Library target required for Nix build tooling (nixLib expects a [lib] target
// when running cargo test --lib during the dependency pre-build phase).

#[macro_use]
extern crate rocket;

mod runner;
pub use runner::{RunError, run};

mod api_error;
mod cli;
mod config;
mod index;
mod ip_range;
mod metrics;
mod ops;
mod ping;
mod register;
mod remove;
mod shell_command_ext;
mod status;
mod unregister;
mod versions;
mod wg;
