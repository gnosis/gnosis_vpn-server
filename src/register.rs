use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::process::Command;

use crate::api_error::ApiError;
use crate::ops::Ops;
use crate::wg::show;

#[derive(Clone, Debug, Serialize)]
pub struct Register {
    public_key: String,
    ip: Ipv4Addr,
    newly_registered: bool,
}

#[derive(Debug, Serialize)]
pub enum Error {
    NoDevice,
    NoFreeIp,
    Generic(String),
    WgShow(show::Error),
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Input {
    public_key: String,
}

#[post("/register", data = "<input>")]
pub fn api(input: Json<Input>, ops: &State<Ops>) -> Result<(Status, Json<Register>), Json<ApiError>> {
    let mut rand = rand::rng();
    let res = run(ops, &mut rand, input.public_key.as_str());

    match res {
        Ok(reg) if reg.newly_registered => Ok((Status::Created, Json(reg))),
        Ok(reg) => Ok((Status::Ok, Json(reg))),
        Err(Error::NoFreeIp) => Err(Json(ApiError::new(404, "Not Found", "No free IP available"))),
        Err(err) => {
            tracing::error!("Error during API register: {:?}", err);
            Err(Json(ApiError::internal_server_error()))
        }
    }
}

pub fn run(ops: &Ops, rng: &mut rand::rngs::ThreadRng, public_key: &str) -> Result<Register, Error> {
    let device = match ops.device() {
        Some(device) => device,
        None => return Err(Error::NoDevice),
    };
    let dump = show::dump(device).map_err(Error::WgShow)?;
    let res_peer = dump.peers.iter().find(|peer| peer.public_key == public_key);
    if let Some(peer) = res_peer {
        return Ok(Register {
            public_key: peer.public_key.clone(),
            ip: peer.ip,
            newly_registered: false,
        });
    }

    let existing_ips: HashSet<Ipv4Addr> = HashSet::from_iter(dump.peers.iter().map(|peer| peer.ip));
    let res_ip = ops.client_address_range.find_free_ip(&existing_ips, rng);
    let ip = match res_ip {
        Some(ip) => ip,
        None => return Err(Error::NoFreeIp),
    };

    let res_output = Command::new("wg")
        .arg("set")
        .arg(device)
        .arg("peer")
        .arg(public_key)
        .arg("allowed-ips")
        .arg(format!("{}/32", ip))
        .output();

    let output = match res_output {
        Ok(output) => output,
        Err(err) => {
            return Err(Error::Generic(format!(
                "wg set peer {} allowed-ips {}/32 failed: {}",
                public_key, ip, err
            )));
        }
    };

    if output.status.success() {
        Ok(Register {
            public_key: public_key.to_string(),
            ip,
            newly_registered: true,
        })
    } else {
        Err(Error::Generic(format!("wg add peer failed: {:?}", output)))
    }
}
