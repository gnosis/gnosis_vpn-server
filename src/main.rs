use clap::Parser;

/// Gnosis VPN server - orchestrate WireGuard server for GnosisVPN connections
#[derive(Parser)]
#[command(version)]
struct Cli {}

fn main() {
    let _args = Cli::parse();
    println!("starting {} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
