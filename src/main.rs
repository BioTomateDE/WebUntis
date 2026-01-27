use anyhow::{Context, Result, ensure};
use chrono::NaiveDate;
use webuntis::api::ApiClient;

struct Args {
    school: String,
    username: String,
    password: String,
    timetable_id: i32,
}

fn collect_args() -> Result<Args> {
    let mut it = std::env::args().skip(1);
    let school = it.next().context("School name missing")?;
    let username = it.next().context("Username missing")?;
    let password = it.next().context("Password missing")?;
    let timetable_id = it.next().context("Timetable ID missing")?;
    let timetable_id = timetable_id
        .parse::<i32>()
        .context("Invalid timetable ID")?;

    ensure!(it.next().is_none(), "Too many arguments");
    Ok(Args {
        school,
        username,
        password,
        timetable_id,
    })
}

fn main() -> Result<()> {
    let args: Args = collect_args()?;

    let client = ApiClient::login(&args.school, &args.username, &args.password)
        .context("Could not request token")?;
    //  let entries = api_client
    //      .fetch_relevant_entry(timetable_id)
    //      .context("Could not fetch timetable entry")?;
    client.fetch_entries(
        NaiveDate::from_ymd_opt(2025, 9, 11).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 30).unwrap(),
        args.timetable_id,
    )?;

    println!("Goodbye, world!");
    Ok(())
}
