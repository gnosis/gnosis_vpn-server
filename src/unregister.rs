use rocket::http::Status;
use rocket::serde::{json::Json, Deserialize};
use rocket::State;
use serde::Serialize;
use std::process::Command;

use crate::ops::Ops;

#[derive(Debug, Serialize)]
pub struct Unregister {
    public_key: String,
}

#[derive(Debug, Serialize)]
pub enum Error {
    NoDevice,
    Generic(String),
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Input {
    public_key: String,
}

#[post("/unregister", data = "<input>")]
pub fn api(input: Json<Input>, ops: &State<Ops>) -> Status {
    let res = run(&ops, input.public_key.as_str());

    match res {
        Ok(_unreg) => Status::NoContent,
        Err(err) => {
            tracing::error!("Error during unregistration: {:?}", err);
            Status::InternalServerError
        }
    }
}

pub fn run(ops: &Ops, public_key: &str) -> Result<Unregister, Error> {
    let device = match ops.device() {
        Some(device) => device,
        None => return Err(Error::NoDevice),
    };

    let res_output = Command::new("wg")
        .arg("set")
        .arg(device)
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

    if output.status.success() {
        Ok(Unregister {
            public_key: public_key.to_string(),
        })
    } else {
        Err(Error::Generic(format!("wg remove peer failed: {:?}", output)))
    }
}
