use std::process;

#[rocket::main]
async fn main() {
    tracing_subscriber::fmt::init();
    if let Err(err) = gnosis_vpn_server::run().await {
        // WgUpFailed/WgDownFailed are logged via tracing::error! inside run().
        // CommandFailed output is already printed to stdout inside run().
        // Fatal errors (config load, etc.) haven't been displayed yet.
        if let gnosis_vpn_server::RunError::Fatal(e) = &err {
            eprintln!("{e:#}");
        }
        process::exit(1);
    }
}
