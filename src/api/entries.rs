use anyhow::{Result, bail};
use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;
use serde_json::Value as JsonValue;

use crate::api::ApiClient;

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
struct Entries {
    _version: FormatVersion,
    days: [Day; 5],
    errors: Vec<JsonValue>,
}

#[derive(Debug, Deserialize)]
struct Day {
    date: NaiveDate,
    status: String,
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
    position1: RowWrapper,
    position2: RowWrapper,
    position3: RowWrapper,
    texts: Vec<EntryText>,
    lesson_text: String,
    lesson_info: String,
    substitution_text: String,
}

#[derive(Debug, Deserialize)]
struct RowWrapper {
    current: Row,
    /// Unstable
    removed: JsonValue,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Row {
    #[serde(rename = "type")]
    row_type: RowType,
    status: Status,
    short_name: String,
    long_name: String,
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct EntryText {
    #[serde(rename = "type")]
    text_type: EntryTextType,
    text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum EntryType {
    NormalTeachingPeriod,
    Event,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum Status {
    Regular,
    Cancelled,
    Changed,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum EntryTextType {
    LessonInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum RowType {
    Subject,
    Teacher,
    Room,
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

        for day in &entries.days {
            println!("Day {:?} - {}", day.date, day.status);
            for entry in &day.grid_entries {
                //println!("  {:?} - {:?}", entry.duration, entry.entry_type, )
                dbg!(entry);
            }
        }

        Ok(entries)
    }
}
