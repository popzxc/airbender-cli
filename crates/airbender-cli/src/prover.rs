use anyhow::{Context, Result};
use execution_utils::unrolled_gpu::{UnrolledProver, UnrolledProverLevel};
use gpu_prover::execution::prover::ExecutionProverConfiguration;
use risc_v_simulator::abstractions::non_determinism::QuasiUARTSource;
use std::fs;
use std::path::Path;
use std::time::Instant;

pub fn prove(
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
