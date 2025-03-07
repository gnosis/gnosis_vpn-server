#[macro_use]
extern crate rocket;

use anyhow::{Context, Result};
use cli::Command;
use figment::providers::{Format, Toml};
use ops::Ops;
use rocket::figment::Figment;
use std::fs;
use std::process;
use tokio::time;

use crate::config::Config;

mod api_error;
mod cli;
mod config;
mod ip_range;
mod ops;
mod register;
mod remove;
mod status;
mod unregister;
mod wg;

#[rocket::main]
async fn main() -> Result<()> {
    let args = cli::parse();

    let config_path = args.config_file;
    let content = fs::read_to_string(config_path).context("failed reading config file")?;
    let config: Config = toml::from_str(&content).context("failed parsing config file content")?;
    let ops = Ops::from(config);

    match args.command {
        Command::Serve {
            periodically_run_cleanup,
        } => {
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
            let rocket = rocket::custom(figment)
                .manage(ops.clone())
                .mount(
                    "/api/v1/clients",
                    routes![register::api, unregister::api, status::api_single],
                )
                .mount("/api/v1", routes![status::api])
                .launch();

            if periodically_run_cleanup {
                tokio::spawn(async move { run_cron(&ops).await });
            }
            rocket.await?;
        }

        Command::Status { json, public_key } if public_key.is_some() => {
            let res = status::run_single(&ops, public_key.unwrap_or("".to_string()).as_str());
            match res {
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
        Command::Status { json, public_key: _ } => {
            let res = status::run(&ops);
            match res {
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

async fn run_cron(ops: &Ops) {
    let mut cron = time::interval(ops.client_cleanup_interval);
    // first tick completes immediately
    cron.tick().await;
    let mut once_not_connected: Vec<String> = Vec::new();
    loop {
        // waits ops.client_cleanup_interval
        cron.tick().await;
        tracing::info!(
            "Running clients cleanup job with {} potential targets from last run",
            once_not_connected.len()
        );
        match remove::cron(ops, &once_not_connected) {
            Ok(newly_not_connected) => {
                once_not_connected = newly_not_connected;
            }
            Err(err) => {
                tracing::error!("Error during clients cleanup: {:?}", err);
                once_not_connected = Vec::new();
            }
        }
    }
}
