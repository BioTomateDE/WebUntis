// TODO: remove this file (or make it actually usable) before publishing

use anyhow::{Context, Result, ensure};
use chrono::NaiveDate;
use webuntis::api::{ApiClient, login};

fn main() -> Result<()> {
    println!("CHello, world!");
    let args: Vec<String> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 4,
        "Provide username, password, school and timetable_id on command line"
    );
    let username = &args[0];
    let password = &args[1];
    let school = &args[2];
    let timetable_id = args[3].parse::<i32>().context("Invalid timetable ID")?;

    let api_client: ApiClient =
        login(username, password, school).context("Could not request token")?;
    //  let entries = api_client
    //      .fetch_relevant_entry(timetable_id)
    //      .context("Could not fetch timetable entry")?;
    api_client.fetch_entries(
        NaiveDate::from_ymd_opt(2025, 9, 11).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 30).unwrap(),
        timetable_id,
    )?;

    println!("Goodbye, world!");
    Ok(())
}
