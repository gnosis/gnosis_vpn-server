use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::collections::HashSet;
use std::net::Ipv4Addr;

use crate::api_error::{self, ApiError};
use crate::ops::Ops;
use crate::wg::{conf, lock, set, show};

#[derive(Clone, Debug, Serialize)]
pub struct Register {
    public_key: String,
    preshared_key: String,
    ip: Ipv4Addr,
    newly_registered: bool,
    server_public_key: String,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("No free IP available")]
    NoFreeIp,
    #[error("IP address already taken")]
    IpAlreadyTaken,
    #[error("wg show error: {0}")]
    WgShow(#[from] show::Error),
    #[error("wg set error: {0}")]
    WgSet(#[from] set::Error),
    #[error("wg lock error: {0}")]
    WgLock(#[from] lock::Error),
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Input {
    public_key: String,
}

pub enum RunVariant {
    GenerateIP(rand::rngs::ThreadRng),
    UseIP(Ipv4Addr),
}

#[post("/register", data = "<input>")]
pub fn api(
    input: Json<Input>,
    sync_wg_interface: &State<bool>,
    ops: &State<Ops>,
) -> Result<(Status, Json<Register>), ApiError> {
    let res = run_locked(ops, **sync_wg_interface, input.public_key.as_str());

    match res {
        Ok(reg) if reg.newly_registered => Ok((Status::Created, Json(reg))),
        Ok(reg) => Ok((Status::Ok, Json(reg))),
        Err(Error::NoFreeIp) => Err(api_error::new(404, "Not Found", "No free IP available")),
        Err(err) => {
            tracing::error!(?err, "POST /register failed");
            Err(api_error::internal_server_error())
        }
    }
}

// Lock spans registration and persisting so no concurrent writer can reassign the claimed IP.
fn run_locked(ops: &Ops, sync_wg_interface: bool, public_key: &str) -> Result<Register, Error> {
    let _wg_lock = lock::acquire(&ops.wg_config)?;
    let register = run(ops, RunVariant::GenerateIP(rand::rng()), public_key)?;

    if register.newly_registered
        && sync_wg_interface
        && let Err(err) = conf::save_file(ops)
    {
        tracing::error!(?err, "Persisting interface state to config failed");
    }

    Ok(register)
}

/// Caller must hold the interface lock, see [`lock::acquire`].
pub fn run(ops: &Ops, variant: RunVariant, public_key: &str) -> Result<Register, Error> {
    let dump = show::dump(ops.interface_name.as_str()).map_err(Error::WgShow)?;
    let res_peer = dump.peers.iter().find(|peer| peer.public_key == public_key);
    if let Some(peer) = res_peer {
        return Ok(Register {
            public_key: peer.public_key.clone(),
            preshared_key: peer.preshared_key.clone(),
            ip: peer.ip,
            newly_registered: false,
            server_public_key: dump.public_key.clone(),
        });
    }

    let existing_ips: HashSet<Ipv4Addr> = HashSet::from_iter(dump.peers.iter().map(|peer| peer.ip));
    let ip = match variant {
        RunVariant::GenerateIP(mut rng) => {
            let res_ip = ops.client_address_range.find_free_ip(&existing_ips, &mut rng);
            match res_ip {
                Some(ip) => ip,
                None => return Err(Error::NoFreeIp),
            }
        }
        RunVariant::UseIP(ip) => {
            if existing_ips.contains(&ip) {
                return Err(Error::IpAlreadyTaken);
            }
            ip
        }
    };

    let preshared_key = set::add_peer(ops.interface_name.as_str(), public_key, &ip).map_err(Error::WgSet)?;
    Ok(Register {
        public_key: public_key.to_string(),
        preshared_key,
        ip,
        newly_registered: true,
        server_public_key: dump.public_key.clone(),
    })
}
