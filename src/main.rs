#[macro_use]
extern crate rocket;

use anyhow::{Context, Result};
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
mod wg_server;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() -> Result<()> {
    let args = cli::parse();

    let config_path = args.config_file;
    let content = fs::read_to_string(config_path).context("failed reading config file")?;
    let config: Config = toml::from_str(&content).context("failed parsing config file")?;
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
            let params = format!(
                r#"
                address = "{address}"
                port = {port}
                "#,
                address = ops.rocket_address,
                port = ops.rocket_port
            );
            let figment = Figment::from(rocket::Config::default()).merge(Toml::string(&params));
            let _rocket = rocket::custom(figment).mount("/", routes![index]).launch().await?;
        }
        Command::Status {} => {
            let device = ops.device().ok_or(anyhow::anyhow!("failed to determine device name"))?;
            let wg_server = wg_server::WgServer::new(&device);
            let dump = wg_server.dump().context("failed to determine wg show dump")?;
            let status = wg_status::new(&dump, &ops);
            println!("status {:?}", status);
        }
    }

    Ok(())
}
