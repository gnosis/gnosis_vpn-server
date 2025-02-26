use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IpRange {
    pub start: Ipv4Addr,
    pub end: Ipv4Addr,
}
