use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

#[derive(Serialize, Deserialize, Debug)]
pub struct IpRange {
    pub start: Ipv4Addr,
    pub end: Ipv4Addr,
}

impl IpRange {
    pub fn new(start: Ipv4Addr, end: Ipv4Addr) -> IpRange {
        IpRange { start, end }
    }
}
