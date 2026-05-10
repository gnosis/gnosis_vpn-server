// Library target required for Nix build tooling (nixLib expects a [lib] target
// when running cargo test --lib during the dependency pre-build phase).

#[macro_use]
extern crate rocket;

pub mod api_error;
pub mod cli;
pub mod config;
pub mod index;
pub mod ip_range;
pub mod metrics;
pub mod ops;
pub mod ping;
pub mod register;
pub mod remove;
pub mod shell_command_ext;
pub mod status;
pub mod unregister;
pub mod versions;
pub mod wg;
