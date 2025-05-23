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
use crate::remove::{RemoveDisconnected, RemoveExpired};

use crate::register::RunVariant;
use crate::wg::conf;
use crate::wg::quick;

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
            sync_wg_interface,
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

            if sync_wg_interface {
                match quick::up(&ops) {
                    Ok(_) => (),
                    Err(err) => {
                        tracing::error!(?err, "Bringing interface up failed");
                        process::exit(1);
                    }
                }
            }

            let figment = Figment::from(rocket::Config::default()).merge(Toml::string(&params));
            let rocket = rocket::custom(figment)
                .manage(ops.clone())
                .manage(sync_wg_interface)
                .mount(
                    "/api/v1/clients",
                    routes![register::api, unregister::api, status::api_single],
                )
                .mount("/api/v1", routes![status::api])
                .launch();

            if periodically_run_cleanup {
                let ops = ops.clone();
                tokio::spawn(async move { run_cron(&ops, sync_wg_interface).await });
            }
            rocket.await?;

            if sync_wg_interface {
                match quick::down(&ops) {
                    Ok(_) => (),
                    Err(err) => {
                        tracing::error!(?err, "Taking interface down failed");
                        process::exit(1);
                    }
                }
            }
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

        Command::Register {
            public_key,
            json,
            force_ip,
            persist_config,
        } => {
            let variant = if let Some(force_ip) = force_ip {
                RunVariant::UseIP(force_ip)
            } else {
                let rand = rand::rng();
                RunVariant::GenerateIP(rand)
            };
            let register = register::run(&ops, variant, &public_key);
            if persist_config {
                if let Err(err) = conf::save_file(&ops) {
                    tracing::error!(?err, "Persisting interface state to config failed");
                }
            }
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

        Command::Unregister {
            public_key,
            json,
            persist_config,
        } => {
            let unregister = unregister::run(&ops, &public_key);
            if persist_config {
                if let Err(err) = conf::save_file(&ops) {
                    tracing::error!(?err, "Persisting interface state to config failed");
                }
            }
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
            persist_config,
        } => {
            let remove_expired = remove::expired(&ops, &client_handshake_timeout_s);
            if persist_config {
                if let Err(err) = conf::save_file(&ops) {
                    tracing::error!(?err, "Persisting interface state to config failed");
                }
            }
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

        Command::RemoveNeverConnected { json, persist_config } => {
            let remove_never_connected = remove::never_connected(&ops);
            if persist_config {
                if let Err(err) = conf::save_file(&ops) {
                    tracing::error!(?err, "Persisting interface state to config failed");
                }
            }
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

async fn run_cron(ops: &Ops, sync_wg_interface: bool) {
    let mut cron = time::interval(ops.client_cleanup_interval);
    // first tick completes immediately
    cron.tick().await;
    let mut once_not_connected: Vec<String> = Vec::new();
    loop {
        // waits ops.client_cleanup_interval
        cron.tick().await;
        tracing::info!(
            "Running clients cleanup job with {} potential never connected targets from last run",
            once_not_connected.len()
        );
        match remove::previously_disconnected(ops, &once_not_connected) {
            Ok(RemoveDisconnected { newly_found, removed }) => {
                tracing::info!("Removed {} clients that were never connected", removed.len());
                once_not_connected = newly_found;
                if sync_wg_interface {
                    match conf::save_file(ops) {
                        Ok(_) => (),
                        Err(err) => {
                            tracing::error!(?err, "Persisting interface state to config failed");
                        }
                    }
                }
            }
            Err(err) => {
                tracing::error!("Error during clients cleanup: {:?}", err);
                once_not_connected = Vec::new();
            }
        }
        match remove::expired(ops, &None) {
            Ok(RemoveExpired { total, .. }) => {
                tracing::info!("Removed {} expired clients", total);
            }
            Err(err) => {
                tracing::error!("Error during expired clients cleanup: {:?}", err);
            }
        }
    }
}
