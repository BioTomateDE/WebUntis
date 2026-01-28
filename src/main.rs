mod logging;

use std::{iter::zip, thread::sleep, time::Duration};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Days, NaiveDate, NaiveTime, Timelike, Utc};
use chrono_tz::Tz;
use clap::Parser;
use reqwest::Url;
use webuntis::{
    LessonInfo,
    discord::DiscordClient,
    extract_all_lessons, send_potential_diffs,
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

    /// The timezone to consider for the dates returned by the Untis API
    #[arg(short = 'z', long, default_value_t = Tz::UTC)]
    timezone: Tz,
}

struct App {
    discord_client: DiscordClient,
    untis_client: UntisClient,
    timetable_id: i32,
    timezone: Tz,
    prev_date: NaiveDate,
    prev_lessons: Option<Vec<LessonInfo>>,
}

impl App {
    #[must_use]
    fn new(
        discord_client: DiscordClient,
        untis_client: UntisClient,
        timetable_id: i32,
        timezone: Tz,
    ) -> Self {
        Self {
            discord_client,
            untis_client,
            timetable_id,
            timezone,
            prev_date: NaiveDate::default(),
            prev_lessons: None,
        }
    }

    fn iteration(&mut self) -> Result<()> {
        log::debug!("Iteration");
        let now: DateTime<Utc> = Utc::now();
        let date: NaiveDate = get_relevant_date(now.with_timezone(&self.timezone));
        let day: Day = self
            .untis_client
            .fetch_single_entry(date, self.timetable_id)?;
        let lessons: Vec<LessonInfo> = extract_all_lessons(&day)?;
        drop(day);

        let Some(prev_lessons) = &self.prev_lessons else {
            self.prev_lessons = Some(lessons);
            self.prev_date = date;
            return Ok(());
        };

        // If it's a different day now, invalidate the "previous day" and rerun.
        if self.prev_date != date {
            self.prev_lessons = None;
            log::info!("Another day, another victory for the OGs.");
            return Ok(());
        }

        if prev_lessons.len() != lessons.len() {
            bail!(
                "Previous and current 'day' have a different number of lessons: {} vs  {}",
                prev_lessons.len(),
                lessons.len(),
            );
        }

        let mut needs_reset: bool = false;
        for (old_lesson, new_lesson) in zip(prev_lessons, &lessons) {
            let sent: bool = send_potential_diffs(&self.discord_client, old_lesson, new_lesson)?;
            if sent {
                needs_reset = true;
            }
        }

        // If there was a change, invalidate the "previous day".
        if needs_reset {
            self.prev_lessons = None;
        }

        let dur = get_sleep_time(now);
        sleep(dur);
        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    logging::init();

    let discord_client = DiscordClient::new(args.discord_webhook_url)
        .context("Could not create Discord Webhook Client")?;

    log::info!("Logging into Untis...");
    let untis_client = UntisClient::login(&args.school, &args.username, &args.password)
        .context("Could not log into Untis")?;

    log::info!("Initialization succeeded!");
    let mut app = App::new(
        discord_client,
        untis_client,
        args.timetable_id,
        args.timezone,
    );

    let mut sequential_errors = 0;

    while sequential_errors < 5 {
        if let Err(e) = app.iteration() {
            app.discord_client.send_error(&e.to_string());
            sequential_errors += 1;
        } else {
            sequential_errors = 0;
        }
    }

    app.discord_client
        .send_error("App failed too many times in a row; shutting down.");
    bail!("App failed {sequential_errors} times in a row");
}

fn get_relevant_date(now: DateTime<Tz>) -> NaiveDate {
    let mut date: NaiveDate = now.date_naive();

    if now.hour() >= 18 {
        // After 18:00, show changes for tomorrow instead of today.
        date = date.checked_add_days(Days::new(1)).unwrap();
    }

    date
}

fn get_sleep_time(now: DateTime<Utc>) -> Duration {
    let time: NaiveTime = now.time();
    let secs = match time.hour() {
        7..8 => 4,
        6..11 => 20,
        11..16 => 40,
        _ => 200,
    };
    Duration::from_secs(secs)
}
