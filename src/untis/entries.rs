mod row;

use std::fmt;

use anyhow::{Context, Result, bail};
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Deserializer};
use serde_json::Value as JsonValue;

use crate::json_util::{parse_datetime, parse_string, parse_vec};
use crate::untis::UntisClient;

// The format version has a custom deserializer to catch errors early in case of format update.
const FORMAT_VERSION: i32 = 19;

#[derive(Debug, Clone, PartialEq, Eq)]
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
        Ok(Self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Duration {
    #[serde(deserialize_with = "parse_datetime")]
    pub start: NaiveDateTime,

    #[serde(deserialize_with = "parse_datetime")]
    pub end: NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct Entries {
    #[allow(unused)]
    format: FormatVersion,
    days: Vec<Day>,
    errors: Vec<JsonValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Day {
    pub date: NaiveDate,
    pub status: Status,
    pub grid_entries: Vec<GridEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridEntry {
    pub duration: Duration,

    #[serde(rename = "type")]
    pub entry_type: EntryType,

    pub status: Status,

    #[serde(deserialize_with = "parse_string")]
    pub notes_all: String,

    #[serde(deserialize_with = "parse_vec")]
    position1: Vec<RowWrapper>,

    #[serde(deserialize_with = "parse_vec")]
    position2: Vec<RowWrapper>,

    #[serde(deserialize_with = "parse_vec")]
    position3: Vec<RowWrapper>,

    #[serde(deserialize_with = "parse_vec")]
    pub texts: Vec<EntryText>,

    #[serde(deserialize_with = "parse_string")]
    pub lesson_text: String,

    #[serde(deserialize_with = "parse_string")]
    pub lesson_info: String,

    #[serde(deserialize_with = "parse_string")]
    pub substitution_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RowWrapper {
    pub current: Option<Row>,
    pub removed: Option<Row>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    #[serde(rename = "type")]
    #[allow(clippy::struct_field_names)]
    pub row_type: RowType,

    pub status: Status,

    #[serde(deserialize_with = "parse_string")]
    pub short_name: String,

    #[serde(deserialize_with = "parse_string")]
    pub long_name: String,

    #[serde(deserialize_with = "parse_string")]
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EntryText {
    #[serde(rename = "type")]
    pub text_type: EntryTextType,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntryType {
    NormalTeachingPeriod,
    Exam,
    Event,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    NoData,
    NotAllowed,
    Regular,
    Added,
    Changed,
    Removed,
    Cancelled,
}

impl Status {
    #[must_use]
    pub const fn is_normal(self) -> bool {
        matches!(self, Self::NoData | Self::NotAllowed | Self::Regular)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Self::NoData => "No Data",
            Self::NotAllowed => "Not Allowed",
            Self::Regular => "Regular",
            Self::Added => "Added",
            Self::Changed => "Changed",
            Self::Removed => "Removed",
            Self::Cancelled => "Cancelled",
        };
        f.write_str(string)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntryTextType {
    LessonInfo,
    SubstitutionText,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RowType {
    Subject,
    Teacher,
    Room,
    Info,
}

impl UntisClient {
    /// Fetch timetable entries from the Untis API between the given dates.
    ///
    /// The range is inclusive on start and end.
    ///
    /// # Errors
    /// There are lots of ways this function can fail:
    /// * Error sending HTTPS request
    /// * Response body is not valid UTF-8
    /// * Server responded with a non-success status code (not 2xx)
    /// * Json deserialization failed
    pub fn fetch_entries(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        timetable_id: i32,
    ) -> Result<Vec<Day>> {
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

        Ok(entries.days)
    }

    pub fn fetch_single_entry(&self, date: NaiveDate, timetable_id: i32) -> Result<Day> {
        let days: Vec<Day> = self
            .fetch_entries(date, date, timetable_id)
            .context("Could not fetch timetable entry")?;

        match days.as_slice() {
            [day] => Ok(day.clone()),
            _ => bail!("API returned {} days instead of just one", days.len()),
        }
    }
}
