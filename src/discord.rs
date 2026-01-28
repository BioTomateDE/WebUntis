pub mod embed;

use std::fmt::Write;

use anyhow::{Context, Result, bail};
use chrono::{Timelike, Utc};
use reqwest::{IntoUrl, Url, blocking::Client};
use serde::Serialize;

use crate::{
    LessonInfo,
    discord::embed::{Color, Embed, Field},
    validate,
};

#[derive(Debug, Clone)]
pub struct DiscordClient {
    http_client: Client,
    url: Url,
}

const LOGO_IMAGE_URL: &str =
    "https://cdn.aptoide.com/imgs/b/1/3/b1399c00075a847dd4e54baddfa11b45_icon.png";

#[derive(Debug, Clone, Serialize)]
struct WebhookRequest<'a> {
    username: &'a str,
    avatar_url: &'a str,
    embeds: Vec<Embed<'a>>,
}

impl DiscordClient {
    pub fn new(webhook_url: impl IntoUrl) -> Result<Self> {
        let url = webhook_url.into_url().context("Invalid WebHook URL")?;
        validate_url(&url).context("Invalid WebHook URL")?;
        let http_client = Client::new();
        Ok(Self { http_client, url })
    }

    pub fn from_parts(id: u64, token: &str) -> Result<Self> {
        let url = format!("https://discord.com/api/webhooks/{id}/{token}");
        Self::new(url)
    }

    fn send_embed(
        &self,
        title: &str,
        content: &str,
        color: Color,
        fields: Vec<Field>,
    ) -> Result<()> {
        let embed = Embed {
            title,
            description: content,
            color,
            timestamp: Utc::now(),
            fields,
        };
        let body = WebhookRequest {
            username: "WebUntis",
            avatar_url: LOGO_IMAGE_URL,
            embeds: vec![embed],
        };
        let resp = self.http_client.post(self.url.clone()).json(&body).send()?;
        resp.error_for_status()?;
        Ok(())
    }

    pub fn send_error(&self, err_message: &str) {
        log::error!("{err_message}");

        let title = "Internal Error";
        let color = Color::new(228, 24, 17);
        if let Err(e) = self.send_embed(title, err_message, color, vec![]) {
            log::error!("Sending error message to webhook failed: {e}");
        }
    }

    pub fn lesson_modification(&self, info: &LessonInfo, title: &str, content: &str) -> Result<()> {
        log::info!(
            "Sending lesson modification regarding {} at {}",
            info.subject,
            info.datetime,
        );

        let time = info.datetime.time();
        let time = format!("{:02}:{:02}", time.hour(), time.minute());

        let fields = vec![
            Field::new("Subject", &info.subject),
            Field::new("Teacher", &info.teacher),
            Field::new("Room", &info.room),
            Field::new("Time", &time),
        ];

        let mut content = format!("({})\n**{}**\n", info.datetime, content);
        let mut push = |a, b| push_content(&mut content, a, b);
        push("Lesson Info", info.lesson_info.as_deref());
        push("Lesson Text", info.lesson_text.as_deref());
        push("Substitution Text", info.substitution_text.as_deref());
        push("Notes", info.notes.as_deref());
        for (i, text) in info.texts.iter().enumerate() {
            let _ = writeln!(content, "**Text #{}:** {}", i + 1, text);
        }

        let color = Color::new(146, 23, 237);
        self.send_embed(title, &content, color, fields)
            .context("sending lesson modification info")
    }
}

fn push_content(buffer: &mut String, label: &'static str, maybe_str: Option<&str>) {
    if let Some(str) = maybe_str {
        let _ = writeln!(buffer, "**{label}:** {str}");
    }
}

fn validate_url(url: &Url) -> Result<()> {
    assert_url_part("Scheme", "https", url.scheme())?;
    assert_url_part("Host", "discord.com", url.host_str().unwrap_or(""))?;
    let segments = url.path_segments().map_or(vec![], |x| x.collect());
    if segments.len() != 4 {
        bail!("Expected 4 URL path segments, got {}", segments.len());
    }
    assert_url_part("Segment #1", "api", segments[0])?;
    assert_url_part("Segment #2", "webhooks", segments[1])?;
    segments[2].parse::<u64>().context("Invalid Webhook ID")?;
    validate::generic_token(segments[3])?;
    if let Some(query) = url.query() {
        bail!("Expected no query, got {query}");
    }
    if let Some(frag) = url.fragment() {
        bail!("Expected no fragment, got {frag}");
    }
    Ok(())
}

fn assert_url_part(label: &'static str, expected: &'static str, actual: &str) -> Result<()> {
    if expected != actual {
        bail!("URL {label} is {actual:?} instead of {expected:?}");
    }
    Ok(())
}
