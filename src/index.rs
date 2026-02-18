use rocket::serde::json::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiInfo {
    pub name: &'static str,
    pub versions: Vec<&'static str>,
    pub latest: &'static str,
}

#[get("/")]
pub fn index() -> Json<ApiInfo> {
    Json(ApiInfo {
        name: "gnosis_vpn-server",
        versions: vec!["v1"],
        latest: "v1",
    })
}
