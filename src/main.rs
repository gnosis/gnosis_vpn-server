#[macro_use]
extern crate rocket;

use anyhow::{Context, Result};
use cli::Command;
use figment::providers::{Format, Toml};
use ops::Ops;
use rocket::figment::Figment;
use std::fs;
use std::process;

use crate::config::Config;

mod cli;
mod config;
mod dump;
mod ip_range;
mod ops;
mod register;
mod remove;
mod status;
mod unregister;

#[rocket::main]
async fn main() -> Result<()> {
    let args = cli::parse();

    let config_path = args.config_file;
    let content = fs::read_to_string(config_path).context("failed reading config file")?;
    let config: Config = toml::from_str(&content).context("failed parsing config file content")?;
    let ops = Ops::from(config);

    match args.command {
        Command::Serve {} => {
            // install global collector configured based on RUST_LOG env var.
            tracing_subscriber::fmt::init();
            tracing::info!(
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
            let _rocket = rocket::custom(figment)
                .manage(ops)
                .mount("/api/v1/clients", routes![register::api])
                .launch()
                .await?;
        }

        Command::Status { json } => {
            let status = status::run(&ops);
            match status {
                Ok(status) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&status)?);
                    } else {
                        println!("{:?}", status);
                    }
                }
                Err(err) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&err)?);
                    } else {
                        println!("{:?}", err);
                    }
                    process::exit(1);
                }
            }
        }

        Command::Register { public_key, json } => {
            let mut rand = rand::rng();
            let register = register::run(&ops, &mut rand, &public_key);
            match register {
                Ok(register) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&register)?);
                    } else {
                        println!("{:?}", register);
                    }
                }
                Err(err) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&err)?);
                    } else {
                        println!("{:?}", err);
                    }
                    process::exit(1);
                }
            }
        }

        Command::Unregister { public_key, json } => {
            let unregister = unregister::run(&ops, &public_key);
            match unregister {
                Ok(_) => {
                    // there is no output for this command
                    if json {
                        println!("{{}}")
                    }
                }
                Err(err) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&err)?);
                    } else {
                        println!("{:?}", err);
                    }
                    process::exit(1);
                }
            }
        }

        Command::RemoveExpired {
            client_handshake_timeout_s,
            json,
        } => {
            let remove_expired = remove::expired(&ops, &client_handshake_timeout_s);
            match remove_expired {
                Ok(remove_expired) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&remove_expired)?);
                    } else {
                        println!("{:?}", remove_expired);
                    }
                }
                Err(err) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&err)?);
                    } else {
                        println!("{:?}", err);
                    }
                    process::exit(1);
                }
            }
        }

        Command::RemoveNeverConnected { json } => {
            let remove_never_connected = remove::never_connected(&ops);
            match remove_never_connected {
                Ok(remove_never_connected) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&remove_never_connected)?);
                    } else {
                        println!("{:?}", remove_never_connected);
                    }
                }
                Err(err) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&err)?);
                    } else {
                        println!("{:?}", err);
                    }
                    process::exit(1);
                }
            }
        }
    }

    Ok(())
}
