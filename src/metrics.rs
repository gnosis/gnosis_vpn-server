use prometheus::{Encoder, IntGauge, Registry, TextEncoder};
use rocket::State;
use rocket::http::ContentType;

use crate::ops::Ops;
use crate::wg::show;

#[derive(Debug, Clone)]
pub struct Metrics {
    pub registered_clients: IntGauge,
}

impl Metrics {
    pub fn new() -> Self {
        let registered_clients = IntGauge::new("gnosisvpn_registered_clients", "Number of registered clients").unwrap(); // New metric

        // Register metrics
        let registry = Registry::default();
        registry.register(Box::new(registered_clients.clone())).unwrap();

        Metrics { registered_clients }
    }

    pub fn gather_metrics(&self) -> String {
        let encoder = TextEncoder::new();
        let registered_metrics = prometheus::gather();
        let mut buffer_metrics = Vec::new();
        encoder.encode(&registered_metrics, &mut buffer_metrics).unwrap();
        String::from_utf8(buffer_metrics).unwrap()
    }
}

pub fn calculate_registered_clients(ops: &Ops) -> u32 {
    let interface = match ops.interface() {
        Some(interface) => interface,
        None => return 0,
    };
    let dump = match show::dump(interface) {
        Ok(dump) => dump,
        Err(_) => return 0,
    };
    dump.peers.len() as u32
}

#[get("/")]
pub fn metrics_endpoint(ops: &State<Ops>) -> (ContentType, String) {
    let registered_clients = calculate_registered_clients(ops);
    ops.metrics.registered_clients.set(registered_clients as i64);
    (ContentType::Plain, ops.metrics.gather_metrics())
}
