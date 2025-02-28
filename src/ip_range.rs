use rand::seq::IndexedRandom;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashSet;
use std::net::Ipv4Addr;

#[derive(Serialize, Clone, Debug)]
pub struct IpRange {
    start: Ipv4Addr,
    end: Ipv4Addr,
}

impl IpRange {
    pub fn count(&self) -> u32 {
        let start = u32::from(self.start);
        let end = u32::from(self.end);
        end - start + 1
    }

    pub fn contains(&self, ip: Ipv4Addr) -> bool {
        let ip = u32::from(ip);
        let start = u32::from(self.start);
        let end = u32::from(self.end);
        ip >= start && ip <= end
    }

    pub fn find_free_ip(&self, taken_ips: &HashSet<Ipv4Addr>, rand: &mut rand::rngs::ThreadRng) -> Option<Ipv4Addr> {
        let start_u32 = u32::from(self.start);
        let end_u32 = u32::from(self.end);

        let available_ips: Vec<Ipv4Addr> = (start_u32..=end_u32)
            .filter(|&ip_u32| !taken_ips.contains(&Ipv4Addr::from(ip_u32)))
            .map(Ipv4Addr::from)
            .collect();

        available_ips.choose(rand).copied()
    }
}

impl<'de> Deserialize<'de> for IpRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct IpRangeHelper {
            start: Ipv4Addr,
            end: Ipv4Addr,
        }

        let helper = IpRangeHelper::deserialize(deserializer)?;

        if helper.start <= helper.end {
            Ok(IpRange {
                start: helper.start,
                end: helper.end,
            })
        } else {
            Err(serde::de::Error::custom(
                "end IP must be greater than or equal to start IP",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let res = toml::from_str::<IpRange>(
            r#"start = "10.128.0.2"
            end = "10.128.0.10""#,
        );
        assert!(res.is_ok());
        let range = res.unwrap();
        assert!(range.start == Ipv4Addr::new(10, 128, 0, 2));
        assert!(range.end == Ipv4Addr::new(10, 128, 0, 10));
        let res2 = toml::from_str::<IpRange>(
            r#"start = "10.128.0.2"
            end = "10.128.0.1""#,
        );
        assert!(res2.is_err());
        let res3 = toml::from_str::<IpRange>(
            r#"start = "10.128.0.2"
            end = "10.12.0.10""#,
        );
        assert!(res3.is_err());
    }
}
