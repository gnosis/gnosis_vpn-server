use rocket::http::Status;
use rocket::serde::{json::Json, Deserialize};
use rocket::State;
use serde::Serialize;
use std::process::Command;

use crate::api_error::ApiError;
use crate::ops::Ops;
use crate::wg::conf;

#[derive(Debug, Serialize)]
pub struct Unregister {
    public_key: String,
}

#[derive(Debug, Serialize)]
pub enum Error {
    NoInterface,
    Generic(String),
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Input {
    public_key: String,
}

#[post("/unregister", data = "<input>")]
pub fn api(input: Json<Input>, sync_wg_interface: &State<bool>, ops: &State<Ops>) -> Result<Status, Json<ApiError>> {
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
            tracing::error!("Error during API unregister: {:?}", err);
            Err(Json(ApiError::internal_server_error()))
        }
    }
}

pub fn run(ops: &Ops, public_key: &str) -> Result<Unregister, Error> {
    let interface = match ops.interface() {
        Some(interface) => interface,
        None => return Err(Error::NoInterface),
    };

    let res_output = Command::new("wg")
        .arg("set")
        .arg(interface)
        .arg("peer")
        .arg(public_key)
        .arg("remove")
        .output();

    let output = match res_output {
        Ok(output) => output,
        Err(err) => {
            return Err(Error::Generic(format!(
                "wg set peer {} remove failed: {}",
                public_key, err
            )));
        }
    };

    if !output.stderr.is_empty() {
        tracing::warn!("wg set peer stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    if output.status.success() {
        Ok(Unregister {
            public_key: public_key.to_string(),
        })
    } else {
        Err(Error::Generic(format!("wg remove peer failed: {:?}", output)))
    }
}
