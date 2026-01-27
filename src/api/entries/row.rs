use anyhow::{Result, bail};

use crate::api::entries::{GridEntry, Row, RowType, RowWrapper};

impl GridEntry {
    pub fn info_maybe_removed(&self) -> Result<(&Row, bool)> {
        extract_one_with_type(&self.position1, RowType::Info)
    }

    pub fn info(&self) -> Result<&Row> {
        ensure_not_removed(self.info_maybe_removed()?)
    }

    pub fn subject_maybe_removed(&self) -> Result<(&Row, bool)> {
        extract_one_with_type(&self.position1, RowType::Subject)
    }

    pub fn subject(&self) -> Result<&Row> {
        ensure_not_removed(self.subject_maybe_removed()?)
    }

    pub fn teacher_maybe_removed(&self) -> Result<(&Row, bool)> {
        extract_one_with_type(&self.position2, RowType::Teacher)
    }

    pub fn teacher(&self) -> Result<&Row> {
        ensure_not_removed(self.teacher_maybe_removed()?)
    }

    pub fn room_maybe_removed(&self) -> Result<(&Row, bool)> {
        extract_one_with_type(&self.position3, RowType::Room)
    }

    pub fn room(&self) -> Result<&Row> {
        ensure_not_removed(self.room_maybe_removed()?)
    }
}

fn extract_one(position_n: &[RowWrapper]) -> Result<&RowWrapper> {
    match position_n.len() {
        0 => bail!("Row is empty"),
        1 => Ok(&position_n[0]),
        n => bail!("Row has {n} elements (expected exactly one)"),
    }
}

fn extract_row_with_status(row_wrapper: &RowWrapper) -> Result<(&Row, bool)> {
    if let Some(current) = &row_wrapper.current {
        Ok((current, false))
    } else if let Some(removed) = &row_wrapper.removed {
        Ok((removed, true)) // Fixed: should be true when removed
    } else {
        bail!("RowWrapper has neither current nor removed Row");
    }
}

fn ensure_not_removed((row, is_removed): (&Row, bool)) -> Result<&Row> {
    if is_removed {
        bail!("Row was removed");
    }
    Ok(row)
}

fn assert_row_type(
    (row, is_removed): (&Row, bool),
    expected_type: RowType,
) -> Result<(&Row, bool)> {
    if row.row_type != expected_type {
        bail!(
            "Expected row type {:?} but got {:?}",
            expected_type,
            row.row_type
        );
    }
    Ok((row, is_removed))
}

fn extract_one_with_type(position: &[RowWrapper], expected_type: RowType) -> Result<(&Row, bool)> {
    let wrapper = extract_one(position)?;
    let (row, is_removed) = extract_row_with_status(wrapper)?;
    assert_row_type((row, is_removed), expected_type)
}
