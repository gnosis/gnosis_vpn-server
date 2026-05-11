#[rocket::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    gnosis_vpn_server::run().await
}
