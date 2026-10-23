use rocket::serde::json::Json;
use serde::{Serialize, Serializer};

use std::fmt::{self, Display};

/// The only API this server speaks; routes mount under its path so label and URL can't drift.
pub const CURRENT: ApiVersion = ApiVersion(1);

/// An exit API label, `"v1"` on the wire.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ApiVersion(u32);

impl ApiVersion {
    pub fn path(self) -> String {
        format!("/api/{self}")
    }
}

impl Display for ApiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.0)
    }
}

impl Serialize for ApiVersion {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

#[derive(Debug, Serialize)]
pub struct Versions {
    pub versions: Vec<ApiVersion>,
    pub latest: ApiVersion,
    pub server: semver::Version,
}

#[get("/")]
pub fn versions() -> Json<Versions> {
    Json(Versions {
        versions: vec![CURRENT],
        latest: CURRENT,
        server: server_version(),
    })
}

fn server_version() -> semver::Version {
    semver::Version::parse(env!("CARGO_PKG_VERSION")).expect("cargo package versions are semver")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_publish_api_labels_and_the_server_version() {
        let json = serde_json::to_value(versions().into_inner()).expect("serialize");

        assert_eq!(json["versions"], serde_json::json!(["v1"]));
        assert_eq!(json["latest"], "v1");
        assert_eq!(json["server"], env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn the_api_path_carries_the_label() {
        assert_eq!(CURRENT.path(), "/api/v1");
    }
}
