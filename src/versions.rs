use rocket::serde::json::Json;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Versions {
    pub versions: Vec<&'static str>,
    pub latest: &'static str,
}

#[get("/")]
pub fn versions() -> Json<Versions> {
    Json(Versions {
        versions: vec!["v1"],
        latest: "v1",
    })
}
