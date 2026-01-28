#![deny(unexpected_cfgs)]
//
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
//
// https://github.com/rust-lang/rust-clippy/issues/16440
#![allow(clippy::multiple_crate_versions)]

use chrono::NaiveDateTime;

use crate::untis::entries::Status;

mod diff;
mod extract;
mod json_util;
mod validate;

pub mod discord;
pub mod untis;

pub use diff::send_potential_diffs;
pub use extract::{extract_all_lessons, extract_lesson_info};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LessonInfo {
    pub status: Status,
    pub datetime: NaiveDateTime,
    pub subject: String,
    pub subject_status: Status,
    pub teacher: String,
    pub teacher_status: Status,
    pub room: String,
    pub room_status: Status,
    pub lesson_info: Option<String>,
    pub lesson_text: Option<String>,
    pub substitution_text: Option<String>,
    pub notes: Option<String>,
    pub texts: Vec<String>,
}
