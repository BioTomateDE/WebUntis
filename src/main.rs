use anyhow::{Context, Result, ensure};
use untis::api::login;

fn main() -> Result<()> {
    println!("Hello, world!");
    let args: Vec<String> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 3,
        "Provide username, password and school on command line"
    );

    let client = login(&args[0], &args[1], &args[2]).context("Could not request token")?;
    dbg!(client.token);

    println!("Goodbye, world!");
    Ok(())
}
