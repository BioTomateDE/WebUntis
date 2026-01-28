use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Field<'a> {
    pub name: &'a str,
    pub value: &'a str,
    pub inline: bool,
}

impl<'a> Field<'a> {
    #[must_use]
    pub const fn new(name: &'a str, value: &'a str) -> Self {
        Self {
            name,
            value,
            inline: true,
        }
    }
}
