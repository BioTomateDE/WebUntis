use anyhow::anyhow;
use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer};

/// Deserializes a Vec, using an empty Vec if the field is null
pub fn parse_vec<'de, D, T>(d: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(d)?.unwrap_or_default())
}

/// Deserializes a String, using an empty String if the field is null
pub fn parse_string<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(d)?.unwrap_or_default())
}

/// Deserializes a [`NaiveDateTime`] using the YYYY-MM-DDThh:mm format
pub fn parse_datetime<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M").map_err(serde::de::Error::custom)
}

pub fn improve_json_error(err: &serde_json::Error, json_string: &str) -> anyhow::Error {
    if err.line() != 1 {
        // Fallback if the JSON is not minified (for some reason)
        return anyhow!("{err}");
    }

    let col = err.column();
    let start = col.saturating_sub(50);
    //let start = col;
    let end = (col + 50).min(json_string.len());
    let start_ell = if start == 0 { "" } else { "..." };
    let end_ell = if end == json_string.len() { "" } else { "..." };

    let snippet = &json_string[start..end];
    anyhow!("{err} | {start_ell}{snippet}{end_ell}")
}
