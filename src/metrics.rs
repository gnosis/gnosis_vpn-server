use prometheus::{IntGauge, Registry, Encoder, TextEncoder};
use rocket::State;

use crate::ops::Ops;

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

#[get("/")]
pub fn metrics_endpoint(ops: &State<Ops>) -> String {
    ops.metrics.gather_metrics()
}
