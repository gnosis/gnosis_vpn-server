#[get("/register")]
pub fn register() -> &'static str {
    "Hello, world!"
}

#[get("/unregister")]
pub fn unregister() -> &'static str {
    "Hello, world2!"
}
