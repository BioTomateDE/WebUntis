use anyhow::Result;

use crate::{
    LessonInfo,
    untis::entries::{Day, GridEntry},
};

pub fn extract_all_lessons(day: &Day) -> Result<Vec<LessonInfo>> {
    day.grid_entries
        .iter()
        .map(extract_lesson_info)
        .filter_map(Result::transpose)
        .collect::<Result<Vec<_>>>()
}

pub fn extract_lesson_info(lesson: &GridEntry) -> Result<Option<LessonInfo>> {
    if lesson.info().is_ok() {
        return Ok(None);
    }

    let subject = lesson.subject()?;
    let (teacher, _) = lesson.teacher_maybe_removed()?;
    let room = lesson.room()?;

    let info = LessonInfo {
        status: lesson.status,
        datetime: lesson.duration.start,
        subject: subject.long_name.clone(),
        subject_status: subject.status,
        teacher: teacher.long_name.clone(),
        teacher_status: teacher.status,
        room: room.long_name.clone(),
        room_status: room.status,
        lesson_info: normalize_str(&lesson.lesson_info),
        lesson_text: normalize_str(&lesson.lesson_text),
        substitution_text: normalize_str(&lesson.substitution_text),
        notes: normalize_str(&lesson.notes_all),
        texts: lesson.texts.iter().map(|x| x.text.clone()).collect(),
    };
    Ok(Some(info))
}

fn normalize_str(string: &str) -> Option<String> {
    let s = string.trim();
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}
