use anyhow::{bail, Context, Result};
use execution_utils::setups;
use execution_utils::unrolled;
use execution_utils::unrolled_gpu::{UnrolledProver, UnrolledProverLevel};
use gpu_prover::execution::prover::ExecutionProverConfiguration;
use risc_v_simulator::abstractions::non_determinism::QuasiUARTSource;
use risc_v_simulator::cycle::IMStandardIsaConfigWithUnsignedMulDiv;
use riscv_transpiler::common_constants::rom::ROM_BYTE_SIZE;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::cli::ProverBackend;
use crate::sim_transpiler;

const DEFAULT_RAM_BOUND_BYTES: usize = 1 << 30;
const DEFAULT_CPU_CYCLE_BOUND: usize = u32::MAX as usize;

pub fn prove(
    app_bin_path: &Path,
    input_words: Vec<u32>,
    output: &Path,
    backend: ProverBackend,
    worker_threads: Option<usize>,
    cycles: Option<usize>,
    ram_bound: Option<usize>,
    level: UnrolledProverLevel,
) -> Result<()> {
    match backend {
        ProverBackend::Gpu => prove_gpu(app_bin_path, input_words, output, worker_threads, level),
        ProverBackend::Cpu => prove_cpu(
            app_bin_path,
            input_words,
            output,
            worker_threads,
            cycles,
            ram_bound,
            level,
        ),
    }
}

fn prove_gpu(
    app_bin_path: &Path,
    input_words: Vec<u32>,
    output: &Path,
    worker_threads: Option<usize>,
    level: UnrolledProverLevel,
) -> Result<()> {
    let prover = create_unrolled_prover(app_bin_path, worker_threads, level)?;
    let oracle = QuasiUARTSource::new_with_reads(input_words);
    tracing::info!("Starting proof generation");
    let start = Instant::now();
    let (proof, cycles) = prover.prove(0, oracle);
    let elapsed = start.elapsed().as_secs_f64();
    tracing::info!("Proof generated in {elapsed:.3}s, cycles={cycles}");
    tracing::info!("{}", proof.debug_info());

    let encoded = bincode::serde::encode_to_vec(&proof, bincode::config::standard())?;
    fs::write(output, encoded)
        .with_context(|| format!("failed to write proof to {}", output.display()))?;
    tracing::info!("Proof written to {}", output.display());
    Ok(())
}

fn prove_cpu(
    app_bin_path: &Path,
    input_words: Vec<u32>,
    output: &Path,
    worker_threads: Option<usize>,
    cycles: Option<usize>,
    ram_bound: Option<usize>,
    level: UnrolledProverLevel,
) -> Result<()> {
    if level != UnrolledProverLevel::Base {
        bail!("CPU backend currently supports only --level base");
    }

    let base_path = strip_bin_suffix(app_bin_path)?;
    let app_bin_path = PathBuf::from(format!("{base_path}.bin"));
    let app_text_path = PathBuf::from(format!("{base_path}.text"));
    if !app_bin_path.exists() {
        bail!("binary not found: {}", app_bin_path.display());
    }
    if !app_text_path.exists() {
        bail!("text file not found: {}", app_text_path.display());
    }

    let (_, binary_u32) = setups::read_and_pad_binary(&app_bin_path);
    let (_, text_u32) = setups::read_and_pad_binary(&app_text_path);

    let cycles_bound = match cycles {
        Some(value) => value,
        None => {
            tracing::info!("Estimating cycles via transpiler (no --cycles provided)");
            let outcome = sim_transpiler::run_transpiler(
                &app_bin_path,
                input_words.clone(),
                DEFAULT_CPU_CYCLE_BOUND,
                Some(&app_text_path),
            )?;
            outcome.cycles_executed
        }
    };

    if cycles_bound == 0 {
        bail!("cycles bound must be greater than 0");
    }

    let ram_bound = ram_bound.unwrap_or(DEFAULT_RAM_BOUND_BYTES);
    if ram_bound < ROM_BYTE_SIZE {
        bail!(
            "ram-bound must be at least {} bytes",
            ROM_BYTE_SIZE
        );
    }

    let threads = worker_threads
        .or_else(|| std::thread::available_parallelism().ok().map(|n| n.get()))
        .unwrap_or(1);
    let worker = execution_utils::prover_examples::prover::worker::Worker::new_with_num_threads(
        threads,
    );

    let oracle = QuasiUARTSource::new_with_reads(input_words);
    tracing::info!(
        "Starting CPU proof generation (cycles_bound={}, ram_bound={})",
        cycles_bound,
        ram_bound
    );
    let start = Instant::now();
    let proof = unrolled::prove_unrolled_for_machine_configuration_into_program_proof::<
        IMStandardIsaConfigWithUnsignedMulDiv,
    >(&binary_u32, &text_u32, cycles_bound, oracle, ram_bound, &worker);
    let elapsed = start.elapsed().as_secs_f64();
    tracing::info!("Proof generated in {elapsed:.3}s");
    tracing::info!("{}", proof.debug_info());

    let encoded = bincode::serde::encode_to_vec(&proof, bincode::config::standard())?;
    fs::write(output, encoded)
        .with_context(|| format!("failed to write proof to {}", output.display()))?;
    tracing::info!("Proof written to {}", output.display());
    Ok(())
}

fn strip_bin_suffix(path: &Path) -> Result<String> {
    let path_str = path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("app path is not valid UTF-8"))?;
    if let Some(stripped) = path_str.strip_suffix(".bin") {
        Ok(stripped.to_string())
    } else {
        Ok(path_str.to_string())
    }
}

fn create_unrolled_prover(
    app_bin_path: &Path,
    worker_threads: Option<usize>,
    level: UnrolledProverLevel,
) -> Result<UnrolledProver> {
    let base_path = strip_bin_suffix(app_bin_path)?;
    let mut configuration = ExecutionProverConfiguration::default();
    if let Some(threads) = worker_threads {
        configuration.max_thread_pool_threads = Some(threads);
        configuration.replay_worker_threads_count = threads;
    }
    Ok(UnrolledProver::new(
        &base_path,
        configuration,
        level,
    ))
}
