use rocket::State;
use rocket::http::Status;
use rocket::serde::{Deserialize, json::Json};
use serde::Serialize;
use thiserror::Error;

use crate::api_error::{self, ApiError};
use crate::ops::Ops;
use crate::wg::{conf, lock, set, show};

#[derive(Debug, Serialize)]
pub struct Unregister {
    public_key: String,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Peer not found")]
    PeerNotFound,
    #[error("wg set error: {0}")]
    WgSet(#[from] set::Error),
    #[error("wg show error: {0}")]
    WgShow(#[from] show::Error),
    #[error("wg lock error: {0}")]
    WgLock(#[from] lock::Error),
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Input {
    public_key: String,
}

#[post("/unregister", data = "<input>")]
pub fn api(input: Json<Input>, sync_wg_interface: &State<bool>, ops: &State<Ops>) -> Result<Status, ApiError> {
    let res = run_locked(ops, **sync_wg_interface, input.public_key.as_str());

    match res {
        Ok(_unreg) => Ok(Status::NoContent),

        Err(Error::PeerNotFound) => Err(api_error::new(404, "Not Found", "Peer not found")),

        Err(err) => {
            tracing::error!(?err, "POST /unregister failed");
            Err(api_error::internal_server_error())
        }
    }
}

// Lock spans removal and persisting so the peer cannot be re-added in between.
fn run_locked(ops: &Ops, sync_wg_interface: bool, public_key: &str) -> Result<Unregister, Error> {
    let _wg_lock = lock::acquire(&ops.wg_config)?;
    let unregister = run(ops, public_key)?;

    if sync_wg_interface && let Err(err) = conf::save_file(ops) {
        tracing::error!(?err, "Persisting interface state to config failed");
    }

    Ok(unregister)
}

/// Caller must hold the interface lock, see [`lock::acquire`].
pub fn run(ops: &Ops, public_key: &str) -> Result<Unregister, Error> {
    let dump = show::dump(ops.interface_name.as_str()).map_err(Error::WgShow)?;
    let res_peer = dump.peers.iter().find(|peer| peer.public_key == public_key);
    let peer = res_peer.ok_or(Error::PeerNotFound)?;
    set::remove_peer(ops.interface_name.as_str(), peer).map_err(Error::WgSet)?;
    Ok(Unregister {
        public_key: public_key.to_string(),
    })
}
