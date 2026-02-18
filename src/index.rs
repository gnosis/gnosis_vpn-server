use rocket::serde::json::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct Index {
    pub name: &'static str,
}

#[get("/")]
pub fn index() -> Json<Index> {
    Json(Index {
        name: "gnosis_vpn-server",
    })
}
