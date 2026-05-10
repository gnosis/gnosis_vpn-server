#[rocket::main]
async fn main() -> anyhow::Result<()> {
    gnosis_vpn_server::run().await
}
