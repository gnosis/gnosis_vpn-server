use prometheus::{Encoder, IntGauge, TextEncoder, register_int_gauge};
use rocket::State;
use rocket::http::ContentType;
use thiserror::Error;

use crate::api_error::{self, ApiError};
use crate::ops::Ops;
use crate::wg::show::{self, Error as ShowError};

#[derive(Debug, Clone)]
pub struct Metrics {
    pub registered_clients: IntGauge,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Prometheus(#[from] prometheus::Error),
    #[error(transparent)]
    Utf8Conversion(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    WgShow(#[from] ShowError),
}

impl Metrics {
    pub fn create() -> Result<Self, Error> {
        let registered_clients = register_int_gauge!("gnosisvpn_registered_clients", "Number of registered clients")?;
        Ok(Metrics { registered_clients })
    }

    pub fn gather_metrics(&self) -> Result<String, Error> {
        let encoder = TextEncoder::new();
        let registered_metrics = prometheus::gather();
        let mut buffer_metrics = Vec::new();
        encoder.encode(&registered_metrics, &mut buffer_metrics)?;
        String::from_utf8(buffer_metrics).map_err(Error::Utf8Conversion)
    }
}

pub fn calculate_registered_clients(ops: &Ops) -> Result<i64, Error> {
    let dump = show::dump(ops.interface_name.as_str())?;
    Ok(dump.peers.len() as i64)
}

#[get("/")]
pub fn metrics_endpoint(ops: &State<Ops>, metrics: &State<Metrics>) -> Result<(ContentType, String), ApiError> {
    let registered_clients = match calculate_registered_clients(ops) {
        Ok(count) => count,
        Err(err) => {
            tracing::error!(?err, "Failed to calculate registered clients");
            return Err(api_error::internal_server_error());
        }
    };
    metrics.registered_clients.set(registered_clients);
    let encoded = match metrics.gather_metrics() {
        Ok(data) => data,
        Err(err) => {
            tracing::error!(?err, "Failed to gather metrics");
            return Err(api_error::internal_server_error());
        }
    };
    Ok((ContentType::Plain, encoded))
}
