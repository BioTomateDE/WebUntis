use anyhow::{Context, Result, ensure};

use untis::api::login::request_new_token;
fn main() -> Result<()> {
    println!("Hello, world!");
    let args: Vec<String> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 3,
        "Provide username, password and school on command line"
    );

    let token =
        request_new_token(&args[0], &args[1], &args[2]).context("Could not request token")?;
    dbg!(token);

    println!("Goodbye, world!");
    Ok(())
}
