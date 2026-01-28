use anyhow::Result;

use crate::{LessonInfo, discord::DiscordClient, untis::entries::Status};

pub fn send_potential_diffs(
    discord: &DiscordClient,
    old: &LessonInfo,
    new: &LessonInfo,
) -> Result<bool> {
    // Cover most common case first
    if old == new {
        return Ok(false);
    }

    if old.status != new.status {
        let body = format!(
            "Lesson Status changed from {} to {}.",
            old.status, new.status,
        );
        if matches!(new.status, Status::Cancelled | Status::Removed) {
            discord.lesson_modification(new, "Lesson Cancellation", &body)?;
        } else if new.status == Status::Changed {
            discord.lesson_modification(new, "Lesson Change", &body)?;
        }
    }

    if old.subject_status != new.subject_status || old.subject != new.subject {
        let body = format!(
            "Subject changed from {} ({}) to {} ({}).",
            old.subject, old.subject_status, new.subject, new.subject_status
        );
        discord.lesson_modification(new, "Subject Changed", &body)?;
    }

    if old.teacher_status != new.teacher_status || old.teacher != new.teacher {
        let body = format!(
            "Teacher changed from {} ({}) to {} ({}).",
            old.teacher, old.teacher_status, new.teacher, new.teacher_status
        );
        discord.lesson_modification(new, "Teacher Changed", &body)?;
    }

    if old.room_status != new.room_status || old.room != new.room {
        let body = format!(
            "Room changed from {} to {} ({}).",
            old.room, new.room, new.room_status
        );
        discord.lesson_modification(new, "Room Changed", &body)?;
    }

    if old.lesson_info != new.lesson_info
        || old.lesson_text != new.lesson_text
        || old.substitution_text != new.substitution_text
        || old.notes != new.notes
        || old.texts != new.texts
    {
        discord.lesson_modification(new, "Notes Changed", "")?;
    }

    Ok(true)
}
