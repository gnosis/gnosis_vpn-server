use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::Ipv4Addr;

use crate::api_error::{self, ApiError};
use crate::ops::Ops;
use crate::wg::conf;
use crate::wg::set;
use crate::wg::show;

#[derive(Clone, Debug, Serialize)]
pub struct Register {
    public_key: String,
    ip: Ipv4Addr,
    newly_registered: bool,
}

#[derive(Debug, Serialize)]
pub enum Error {
    NoInterface,
    NoFreeIp,
    IpAlreadyTaken,
    WgShow(show::Error),
    WgSet(set::Error),
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
    let rand = rand::rng();
    let res = run(ops, RunVariant::GenerateIP(rand), input.public_key.as_str());

    match res {
        Ok(reg) if reg.newly_registered => {
            if **sync_wg_interface {
                match conf::save_file(ops) {
                    Ok(_) => (),
                    Err(err) => {
                        tracing::error!(?err, "Persisting interface state to config failed");
                    }
                }
            }

            Ok((Status::Created, Json(reg)))
        }
        Ok(reg) => Ok((Status::Ok, Json(reg))),
        Err(Error::NoFreeIp) => Err(api_error::new(404, "Not Found", "No free IP available")),
        Err(err) => {
            tracing::error!(?err, "POST /register failed");
            Err(api_error::internal_server_error())
        }
    }
}

pub fn run(ops: &Ops, variant: RunVariant, public_key: &str) -> Result<Register, Error> {
    let interface = match ops.interface() {
        Some(interface) => interface,
        None => return Err(Error::NoInterface),
    };
    let dump = show::dump(interface).map_err(Error::WgShow)?;
    let res_peer = dump.peers.iter().find(|peer| peer.public_key == public_key);
    if let Some(peer) = res_peer {
        return Ok(Register {
            public_key: peer.public_key.clone(),
            ip: peer.ip,
            newly_registered: false,
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

    set::add_peer(interface, public_key, &ip).map_err(Error::WgSet)?;
    Ok(Register {
        public_key: public_key.to_string(),
        ip,
        newly_registered: true,
    })
}
