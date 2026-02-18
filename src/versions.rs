use rocket::serde::json::Json;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Versions {
    pub versions: Vec<String>,
}

#[get("/versions")]
pub fn api_versions() -> Json<Versions> {
    Json(Versions {
        versions: vec![
            "v1".to_string(),
        ],
    })
}
