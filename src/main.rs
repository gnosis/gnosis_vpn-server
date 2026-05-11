use std::process;

#[rocket::main]
async fn main() {
    tracing_subscriber::fmt::init();
    if let Err(err) = gnosis_vpn_server::run().await {
        eprintln!("{err:#}");
        process::exit(1);
    }
}
