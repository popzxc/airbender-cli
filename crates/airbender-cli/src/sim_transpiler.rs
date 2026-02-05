use anyhow::{Result, bail};
use risc_v_simulator::abstractions::non_determinism::QuasiUARTSource;
use riscv_transpiler::common_constants::{INITIAL_TIMESTAMP, TIMESTAMP_STEP};
use riscv_transpiler::jit::JittedCode;
use std::path::{Path, PathBuf};
use tracing::warn;

use crate::sim::SimulationOutcome;

pub fn run_transpiler(
    bin_path: &Path,
    input_words: Vec<u32>,
    cycles: usize,
    text_path: Option<&PathBuf>,
) -> Result<SimulationOutcome> {
    if !bin_path.exists() {
        bail!("binary not found: {}", bin_path.display());
    }
    let text_path = text_path
        .cloned()
        .unwrap_or_else(|| derive_text_path(bin_path));
    if !text_path.exists() {
        bail!("text file not found: {}", text_path.display());
    }

    let bin_words = read_u32_words(bin_path)?;
    let text_words = read_u32_words(&text_path)?;

    let mut non_determinism_source = QuasiUARTSource::new_with_reads(input_words);

    let cycles_bound = match u32::try_from(cycles) {
        Ok(value) => Some(value),
        Err(_) => {
            warn!(
                "Cycles limit {} exceeds u32; running without a cycle bound",
                cycles
            );
            None
        }
    };

    let (state, _memory) = JittedCode::run_alternative_simulator(
        &text_words,
        &mut non_determinism_source,
        &bin_words,
        cycles_bound,
    );

    let cycles_executed = ((state.timestamp - INITIAL_TIMESTAMP) / TIMESTAMP_STEP) as usize;

    Ok(SimulationOutcome {
        registers: state.registers,
        cycles_executed,
        reached_end: true,
    })
}

fn derive_text_path(bin_path: &Path) -> PathBuf {
    let mut text_path = bin_path.to_path_buf();
    text_path.set_extension("text");
    text_path
}

fn read_u32_words(path: &Path) -> Result<Vec<u32>> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;
    if buffer.len() % 4 != 0 {
        bail!("file length is not a multiple of 4: {}", path.display());
    }
    let mut words = Vec::with_capacity(buffer.len() / 4);
    for chunk in buffer.as_chunks::<4>().0 {
        words.push(u32::from_le_bytes(*chunk));
    }
    Ok(words)
}
