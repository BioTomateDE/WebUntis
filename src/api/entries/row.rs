use anyhow::{Result, bail};

use crate::api::entries::{GridEntry, Row, RowType, RowWrapper};

impl GridEntry {
    pub fn subject_maybe_removed(&self) -> Result<(&Row, bool)> {
        maybe_removed(extract_one(&self.position1)?).and_then(assert_type(RowType::Subject))
    }

    pub fn subject(&self) -> Result<&Row> {
        not_removed(self.subject_maybe_removed()?)
    }

    pub fn teacher_maybe_removed(&self) -> Result<(&Row, bool)> {
        maybe_removed(extract_one(&self.position2)?).and_then(assert_type(RowType::Teacher))
    }

    pub fn teacher(&self) -> Result<&Row> {
        not_removed(self.teacher_maybe_removed()?)
    }

    pub fn room_maybe_removed(&self) -> Result<(&Row, bool)> {
        maybe_removed(extract_one(&self.position3)?).and_then(assert_type(RowType::Room))
    }

    pub fn room(&self) -> Result<&Row> {
        not_removed(self.room_maybe_removed()?)
    }
}

fn extract_one(position_n: &[RowWrapper]) -> Result<&RowWrapper> {
    if position_n.is_empty() {
        bail!("Row is empty");
    }
    let n = position_n.len();
    if n > 1 {
        bail!("Row has {n} elements (more than one)");
    }
    Ok(&position_n[0])
}

fn maybe_removed(row_wrapper: &RowWrapper) -> Result<(&Row, bool)> {
    if let Some(cur) = &row_wrapper.current {
        Ok((cur, false))
    } else if let Some(rem) = &row_wrapper.removed {
        Ok((rem, false))
    } else {
        bail!("RowWrapper neither has current nor removed Row");
    }
}

fn not_removed(tuple: (&Row, bool)) -> Result<&Row> {
    let (subject, is_removed) = tuple;
    if is_removed {
        bail!("Subject was removed");
    }
    Ok(subject)
}

// this is the dumbest function i've ever written xd
fn assert_type(ty: RowType) -> impl FnOnce((&Row, bool)) -> Result<(&Row, bool)> {
    move |tuple: (&Row, bool)| {
        let rty = tuple.0.row_type;
        if rty != ty {
            bail!("Expected row type {ty:?} but got {rty:?}")
        }
        Ok(tuple)
    }
}

// TODO:  this function doesnt work because it has to use `removed` otherwise
