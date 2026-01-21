use anyhow::{Result, bail};
use chrono::{DateTime, Days, NaiveDate, NaiveDateTime, Timelike, Utc};
use serde::{Deserialize, Deserializer};
use serde_json::Value as JsonValue;

use crate::api::ApiClient;

// The format version has a custom deserializer to catch errors early in case of format update.
const FORMAT_VERSION: i32 = 19;

#[derive(Debug)]
struct FormatVersion;

impl<'de> Deserialize<'de> for FormatVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
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

/// Deserializes a Vec, using an empty Vec if the field is null
fn parse_vec<'de, D, T>(d: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(d)?.unwrap_or_default())
}

/// Deserializes a String, using an empty String if the field is null
fn parse_string<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(d)?.unwrap_or_default())
}

fn parse_datetime<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M").map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize)]
struct Duration {
    #[serde(deserialize_with = "parse_datetime")]
    start: NaiveDateTime,

    #[serde(deserialize_with = "parse_datetime")]
    end: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct Entries {
    #[expect(unused)]
    format: FormatVersion,
    days: Vec<Day>,
    errors: Vec<JsonValue>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Day {
    date: NaiveDate,
    status: Status,
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

    #[serde(deserialize_with = "parse_string")]
    notes_all: String,

    #[serde(deserialize_with = "parse_vec")]
    position1: Vec<RowWrapper>,

    #[serde(deserialize_with = "parse_vec")]
    position2: Vec<RowWrapper>,

    #[serde(deserialize_with = "parse_vec")]
    position3: Vec<RowWrapper>,

    #[serde(deserialize_with = "parse_vec")]
    texts: Vec<EntryText>,

    #[serde(deserialize_with = "parse_string")]
    lesson_text: String,

    #[serde(deserialize_with = "parse_string")]
    lesson_info: String,

    #[serde(deserialize_with = "parse_string")]
    substitution_text: String,
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

    #[serde(deserialize_with = "parse_string")]
    short_name: String,

    #[serde(deserialize_with = "parse_string")]
    long_name: String,

    #[serde(deserialize_with = "parse_string")]
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct EntryText {
    #[serde(rename = "type")]
    text_type: EntryTextType,
    text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum EntryType {
    NormalTeachingPeriod,
    Exam,
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
