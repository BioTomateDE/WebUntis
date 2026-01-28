mod color;
mod field;

use chrono::{DateTime, Utc};
use serde::Serialize;

pub use color::Color;
pub use field::Field;

#[derive(Debug, Clone, Serialize)]
pub struct Embed<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub color: Color,
    pub timestamp: DateTime<Utc>,
    pub fields: Vec<Field<'a>>,
}
