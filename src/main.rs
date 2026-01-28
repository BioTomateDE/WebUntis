use anyhow::{Context, Result};
use clap::Parser;
use reqwest::Url;
use webuntis::{
    LessonInfo,
    discord::DiscordClient,
    extract_all_lessons, extract_lesson_info, send_potential_diffs,
    untis::{UntisClient, entries::Day},
};

/// WebUntis Notification Bot
#[derive(Parser)]
struct Args {
    /// Subdomain Name of the school
    #[arg(short, long)]
    school: String,

    /// Your WebUntis username
    #[arg(short, long)]
    username: String,

    /// Your WebUntis password
    #[arg(short, long)]
    password: String,

    /// The Timetable ID (aka `resources` in json)
    #[arg(short, long)]
    timetable_id: i32,

    /// The Discord WebHook URL the notifications should be sent to
    #[arg(short, long)]
    discord_webhook_url: Url,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let untis_client = UntisClient::login(&args.school, &args.username, &args.password)
        .context("Could not create Untis Client")?;

    let discord_client = DiscordClient::new(args.discord_webhook_url)
        .context("Could not create Discord Webhook Client")?;

    let day: Day = untis_client
        .fetch_relevant_entry(args.timetable_id)
        .context("Could not fetch timetable entry")?;

    let lessons: Vec<LessonInfo> = extract_all_lessons(&day)?;
    drop(day);

    for lesson in &lessons {
        send_potential_diffs(&discord_client, lesson, lesson)?;
    }

    println!("Goodbye, world!");
    Ok(())
}
