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
pub struct ApiError {
    error: InternalApiError,
}

impl ApiError {
    pub fn new(code: u16, reason: &str, description: &str) -> Self {
        Self {
            error: InternalApiError {
                code,
                reason: reason.to_string(),
                description: description.to_string(),
            },
        }
    }

    pub fn internal_server_error() -> Self {
        Self {
            error: InternalApiError {
                code: 500,
                reason: "Internal Server Error".to_string(),
                description: "The server encountered an internal error while processing this request.".to_string(),
            },
        }
    }
}
