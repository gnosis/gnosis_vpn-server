#[macro_use]
extern crate rocket;

use anyhow::Result;
use cli::Command;
use figment::providers::{Format, Toml};
use ops::Ops;
use rocket::figment::Figment;
use std::fs;

use crate::config::Config;

mod cli;
mod config;
mod ip_range;
mod ops;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() -> Result<()> {
    let args = cli::parse();

    let config_path = args.config_file;
    let content = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&content)?;
    let ops = Ops::from(config);

    match args.command {
        Command::Serve {} => {
            println!(
                "serving {name} v{version} on {ip}:{port}",
                name = env!("CARGO_PKG_NAME"),
                version = env!("CARGO_PKG_VERSION"),
                ip = ops.rocket_address,
                port = ops.rocket_port
            );
            let params = r"
                address = {ops.rocket_address}
                port = {ops.rocket_port}
            ";
            let figment = Figment::from(rocket::Config::default()).merge(Toml::string(params));
            let _rocket = rocket::custom(figment).mount("/", routes![index]).launch().await?;
        }
    }

    Ok(())
}
