use anyhow::{Context, Result, bail};
use chrono::{DateTime, Days, NaiveDate, NaiveDateTime, Timelike, Utc};
use serde::Deserialize;
use serde_json::Value as JsonValue;

use crate::api::ApiClient;

// The format version has a custom deserializer to catch errors early in case of format update.
const FORMAT_VERSION: i32 = 19;

#[derive(Debug)]
struct FormatVersion;

impl<'de> Deserialize<'de> for FormatVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let version = i32::deserialize(deserializer)?;
        if version != FORMAT_VERSION {
            return Err(serde::de::Error::custom(format!(
                "Format Version mismatch: expected {FORMAT_VERSION}, got {version} (contact project maintainers)"
            )));
        }
        Ok(FormatVersion)
    }
}

fn deserialize_naive_datetime<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M").map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize)]
struct Duration {
    #[serde(deserialize_with = "deserialize_naive_datetime")]
    start: NaiveDateTime,

    #[serde(deserialize_with = "deserialize_naive_datetime")]
    end: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct Entries {
    #[expect(unused)]
    format: FormatVersion,
    #[serde(default)]
    days: Vec<Day>,
    #[serde(default)]
    errors: Vec<JsonValue>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Day {
    date: NaiveDate,
    status: String,
    #[serde(default)]
    grid_entries: Vec<GridEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GridEntry {
    duration: Duration,
    #[serde(rename = "type")]
    entry_type: EntryType,
    /// Unstable
    status_detail: JsonValue,
    /// Unstable
    name: JsonValue,
    notes_all: String,
    #[serde(default)]
    position1: Vec<RowWrapper>,
    #[serde(default)]
    position2: Vec<RowWrapper>,
    #[serde(default)]
    position3: Vec<RowWrapper>,
    #[serde(default)]
    texts: Vec<EntryText>,
    lesson_text: Option<String>,
    lesson_info: Option<String>,
    substitution_text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RowWrapper {
    current: Option<Row>,
    /// Unstable
    removed: JsonValue,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Row {
    #[serde(rename = "type")]
    row_type: Option<RowType>,
    status: Option<Status>,
    short_name: Option<String>,
    long_name: Option<String>,
    display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EntryText {
    #[serde(rename = "type")]
    text_type: EntryTextType,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum EntryType {
    NormalTeachingPeriod,
    Event,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Status {
    NoData,
    NotAllowed,
    Regular,
    Added,
    Changed,
    Cancelled,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum EntryTextType {
    LessonInfo,
    SubstitutionText,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum RowType {
    Subject,
    Teacher,
    Room,
    Info,
}

impl ApiClient {
    pub fn fetch_entries(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        timetable_id: i32,
    ) -> Result<Entries> {
        let query: &[(&str, &str)] = &[
            ("start", &start.to_string()),
            ("end", &end.to_string()),
            ("resourceType", "CLASS"), // can be changed to STUDENT
            ("resources", &timetable_id.to_string()),
            ("format", &FORMAT_VERSION.to_string()),
        ];

        let entries: Entries = self.get_json("timetable/entries", query)?;

        if !entries.errors.is_empty() {
            bail!("API returned errors: {:?}", entries.errors);
        }

        // for day in &entries.days {
        //     println!("Day {:?} - {}", day.date, day.status);
        //     for entry in &day.grid_entries {
        //         //println!("  {:?} - {:?}", entry.duration, entry.entry_type, )
        //         dbg!(entry);
        //     }
        // }

        Ok(entries)
    }

    pub fn fetch_relevant_entry(&self, timetable_id: i32) -> Result<Entries> {
        // NOTE: This will only behave properly for schools near GMT+0
        let now: DateTime<Utc> = Utc::now();
        let mut date: NaiveDate = now.date_naive();
        if now.hour() >= 18 {
            // After 18:00, show changes for tomorrow instead of today.
            date = date.checked_add_days(Days::new(1)).unwrap();
        }
        self.fetch_entries(date, date, timetable_id)
    }
}
