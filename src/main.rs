#[macro_use]
extern crate rocket;

use cli::Command;

mod cli;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let args = cli::parse();

    match args.command {
        Command::Serve {} => {
            println!("starting {} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

            let _rocket = rocket::build().mount("/", routes![index]).launch().await?;
        }
    }

    Ok(())
}
