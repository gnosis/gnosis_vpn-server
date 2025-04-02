use rocket::http::Status;
use rocket::serde::{json::Json, Deserialize};
use rocket::State;
use serde::Serialize;

use crate::api_error::{self, ApiError};
use crate::ops::Ops;
use crate::wg::{conf, set, show};

#[derive(Debug, Serialize)]
pub struct Unregister {
    public_key: String,
}

#[derive(Debug, Serialize)]
pub enum Error {
    NoInterface,
    PeerNotFound,
    WgSet(set::Error),
    WgShow(show::Error),
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Input {
    public_key: String,
}

#[post("/unregister", data = "<input>")]
pub fn api(input: Json<Input>, sync_wg_interface: &State<bool>, ops: &State<Ops>) -> Result<Status, ApiError> {
    let res = run(ops, input.public_key.as_str());

    match res {
        Ok(_unreg) => {
            if **sync_wg_interface {
                match conf::save_file(ops) {
                    Ok(_) => (),
                    Err(err) => {
                        tracing::error!(?err, "Persisting interface state to config failed");
                    }
                }
            }

            Ok(Status::NoContent)
        }

        Err(err) => {
            tracing::error!(?err, "POST /unregister failed");
            Err(api_error::internal_server_error())
        }
    }
}

pub fn run(ops: &Ops, public_key: &str) -> Result<Unregister, Error> {
    let interface = match ops.interface() {
        Some(interface) => interface,
        None => return Err(Error::NoInterface),
    };
    let dump = show::dump(interface).map_err(Error::WgShow)?;
    let res_peer = dump.peers.iter().find(|peer| peer.public_key == public_key);
    let peer = res_peer.ok_or(Error::PeerNotFound)?;
    set::remove_peer(interface, &peer).map_err(Error::WgSet)?;
    Ok(Unregister {
        public_key: public_key.to_string(),
    })
}
