use rocket::http::Status;
use rocket::serde::json::Json;
use serde::Serialize;

/*
 * keep consistent with default:
   {
     "error": {
        "code": 500,
        "reason": "Internal Server Error",
        "description": "The server encountered an internal error while processing this request."
    }
*/
#[derive(Serialize)]
struct InternalApiError {
    code: u16,
    reason: String,
    description: String,
}

#[derive(Serialize)]
pub struct JsonApiError {
    error: InternalApiError,
}

pub type ApiError = (Status, Json<JsonApiError>);

pub fn new(code: u16, reason: &str, description: &str) -> ApiError {
    (
        Status::from_code(code).unwrap_or(Status::InternalServerError),
        Json(JsonApiError {
            error: InternalApiError {
                code,
                reason: reason.to_string(),
                description: description.to_string(),
            },
        }),
    )
}

pub fn internal_server_error() -> ApiError {
    (
        Status::InternalServerError,
        Json(JsonApiError {
            error: InternalApiError {
                code: 500,
                reason: "Internal Server Error".to_string(),
                description: "The server encountered an internal error while processing this request.".to_string(),
            },
        }),
    )
}
