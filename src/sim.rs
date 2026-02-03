use anyhow::{Result, bail};
use risc_v_simulator::abstractions::non_determinism::QuasiUARTSource;
use risc_v_simulator::cycle::IMStandardIsaConfig;
use risc_v_simulator::runner::CUSTOM_ENTRY_POINT;
use risc_v_simulator::setup::BaselineWithND;
use risc_v_simulator::sim::{
    BinarySource, DiagnosticsConfig, ProfilerConfig, Simulator, SimulatorConfig,
};
use std::path::{Path, PathBuf};

pub const DEFAULT_CYCLES: usize = 90_000_000_000;

#[derive(Debug)]
pub struct SimulationOutcome {
    pub registers: [u32; 32],
    pub cycles_executed: usize,
    pub reached_end: bool,
}

pub fn profiler_diagnostics(
    app_bin: &Path,
    elf_path: Option<PathBuf>,
    output: PathBuf,
    sampling_rate: usize,
    inverse: bool,
) -> Result<DiagnosticsConfig> {
    let symbols_path = elf_path.unwrap_or_else(|| derive_elf_path(app_bin));
    if !symbols_path.exists() {
        bail!("ELF file not found: {}", symbols_path.display());
    }

    let mut diagnostics = DiagnosticsConfig::new(symbols_path);
    let mut profiler = ProfilerConfig::new(output);
    profiler.frequency_recip = sampling_rate;
    profiler.reverse_graph = inverse;
    diagnostics.profiler_config = Some(profiler);
    Ok(diagnostics)
}

pub fn run_simulator(
    bin_path: &Path,
    input_words: Vec<u32>,
    cycles: usize,
    diagnostics: Option<DiagnosticsConfig>,
) -> Result<SimulationOutcome> {
    if !bin_path.exists() {
        bail!("binary not found: {}", bin_path.display());
    }
    let config = SimulatorConfig::new(
        BinarySource::Path(bin_path.to_path_buf()),
        CUSTOM_ENTRY_POINT,
        cycles,
        diagnostics,
    );
    let non_determinism_source = QuasiUARTSource::new_with_reads(input_words);
    let setup = BaselineWithND::<_, IMStandardIsaConfig>::new(non_determinism_source);
    let mut sim = Simulator::<_, IMStandardIsaConfig>::new(config, setup);
    let mut last_cycle = 0usize;
    let result = sim.run(|_, _| {}, |_, cycle| last_cycle = cycle);
    let cycles_executed = if result.reached_end {
        last_cycle.saturating_add(1)
    } else {
        cycles
    };

    Ok(SimulationOutcome {
        registers: result.state.registers,
        cycles_executed,
        reached_end: result.reached_end,
    })
}

pub fn report_run_outcome(outcome: &SimulationOutcome) {
    tracing::info!(
        "Execution finished: cycles_executed: {}, reached_end: {}",
        outcome.cycles_executed,
        outcome.reached_end
    );
    let mut registers_str = String::new();
    for (idx, value) in outcome.registers[10..18].iter().enumerate() {
        registers_str.push_str(&format!("x{}={} ", 10 + idx, value));
    }
    tracing::info!("Output values: {}", registers_str.trim());
}

fn derive_elf_path(bin_path: &Path) -> PathBuf {
    let mut elf_path = bin_path.to_path_buf();
    elf_path.set_extension("elf");
    elf_path
}
