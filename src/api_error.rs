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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_build_custom_error_payload_with_explicit_status() -> anyhow::Result<()> {
        let (status, Json(body)) = new(404, "Not Found", "Client missing");
        assert_eq!(status, Status::NotFound);
        assert_eq!(body.error.code, 404);
        assert_eq!(body.error.reason, "Not Found");
        assert_eq!(body.error.description, "Client missing");

        Ok(())
    }

    #[test]
    fn should_use_internal_server_error_when_status_unrecognized() -> anyhow::Result<()> {
        let (status, Json(body)) = new(999, "Weird", "Unexpected");

        assert_eq!(status, Status::InternalServerError);
        assert_eq!(body.error.code, 999);
        assert_eq!(body.error.reason, "Weird");
        assert_eq!(body.error.description, "Unexpected");

        Ok(())
    }

    #[test]
    fn should_return_consistent_default_internal_server_error() -> anyhow::Result<()> {
        let (status, Json(body)) = internal_server_error();
        assert_eq!(status, Status::InternalServerError);
        assert_eq!(body.error.code, 500);
        assert_eq!(body.error.reason, "Internal Server Error");

        Ok(())
    }
}
