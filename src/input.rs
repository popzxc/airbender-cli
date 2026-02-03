use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;

pub fn parse_input_words(path: &Path) -> Result<Vec<u32>> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read input file {}", path.display()))?;
    let mut hex: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    if let Some(stripped) = hex.strip_prefix("0x") {
        hex = stripped.to_string();
    }

    if hex.is_empty() {
        return Ok(Vec::new());
    }
    if !hex.len().is_multiple_of(8) {
        bail!(
            "input hex length must be a multiple of 8 (got {})",
            hex.len()
        );
    }

    let mut words = Vec::with_capacity(hex.len() / 8);
    for chunk in hex.as_bytes().chunks(8) {
        let chunk_str = std::str::from_utf8(chunk).context("input is not valid UTF-8")?;
        let word = u32::from_str_radix(chunk_str, 16)
            .with_context(|| format!("invalid hex word: {chunk_str}"))?;
        words.push(word);
    }
    Ok(words)
}
