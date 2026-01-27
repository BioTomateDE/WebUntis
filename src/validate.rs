use std::collections::HashSet;

use anyhow::{Context, Result, bail, ensure};

pub fn school(school_name: &str) -> Result<()> {
    const CHARS: &[u8; 27] = b"abcdefghijklmnopqrstuvwxyz-";
    validate_charset("School Name", school_name, CHARS)
}

pub fn token(token: &str) -> Result<()> {
    const CHARS: &[u8; 65] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.";
    let mut parts = token.split('.');
    for _ in 0..3 {
        let part: &str = parts.next().context("Token has too few parts")?;
        validate_charset("Token", part, CHARS)?;
    }
    ensure!(parts.next().is_none(), "Token has too many parts");
    Ok(())
}

fn validate_charset(description: &'static str, string: &str, charset: &'static [u8]) -> Result<()> {
    if string.is_empty() {
        bail!("{description} is empty");
    }
    if string.bytes().all(|b| charset.contains(&b)) {
        return Ok(());
    }

    let set = string
        .bytes()
        .filter(|b| !charset.contains(b))
        .map(char::from)
        .collect::<HashSet<char>>();
    bail!("{description} contains invalid characters: {set:?}");
}
