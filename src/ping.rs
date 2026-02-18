use rocket::http::Status;

#[get("/ping")]
pub fn ping() -> Status {
    Status::NoContent
}
