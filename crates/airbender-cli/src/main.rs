use anyhow::Result;
use clap::Parser;
use execution_utils::unrolled_gpu::UnrolledProverLevel;

mod cli;
mod input;
mod prover;
mod sim;
mod sim_transpiler;
mod vk;

fn main() -> Result<()> {
    init_tracing()?;
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Run {
            app_bin,
            input,
            cycles,
        } => {
            let input_words = input::parse_input_words(&input)?;
            let cycle_limit = cycles.unwrap_or(sim::DEFAULT_CYCLES);
            tracing::info!("Running simulator");
            let outcome = sim::run_simulator(&app_bin, input_words, cycle_limit, None)?;
            sim::report_run_outcome(&outcome);
        }
        cli::Commands::Flamegraph {
            app_bin,
            input,
            output,
            cycles,
            sampling_rate,
            inverse,
            elf_path,
        } => {
            let input_words = input::parse_input_words(&input)?;
            let cycle_limit = cycles.unwrap_or(sim::DEFAULT_CYCLES);
            let diagnostics =
                sim::profiler_diagnostics(&app_bin, elf_path, output, sampling_rate, inverse)?;
            tracing::info!("Running simulator with profiler");
            let outcome =
                sim::run_simulator(&app_bin, input_words, cycle_limit, Some(diagnostics))?;
            sim::report_run_outcome(&outcome);
        }
        cli::Commands::RunTranspiler {
            app_bin,
            input,
            cycles,
            text_path,
        } => {
            let input_words = input::parse_input_words(&input)?;
            let cycle_limit = cycles.unwrap_or(sim::DEFAULT_CYCLES);
            tracing::info!("Running transpiler JIT");
            let outcome = sim_transpiler::run_transpiler(
                &app_bin,
                input_words,
                cycle_limit,
                text_path.as_ref(),
            )?;
            sim::report_run_outcome(&outcome);
        }
        cli::Commands::Prove {
            app_bin,
            input,
            output,
            backend,
            threads,
            cycles,
            ram_bound,
            level,
        } => {
            let input_words = input::parse_input_words(&input)?;
            let prover_level = match level {
                cli::ProverLevel::Base => UnrolledProverLevel::Base,
                cli::ProverLevel::RecursionUnrolled => UnrolledProverLevel::RecursionUnrolled,
                cli::ProverLevel::RecursionUnified => UnrolledProverLevel::RecursionUnified,
            };
            prover::prove(
                &app_bin,
                input_words,
                &output,
                backend,
                threads,
                cycles,
                ram_bound,
                prover_level,
            )?;
        }
        cli::Commands::GenerateVk {
            app_bin,
            output,
            level,
        } => {
            let prover_level = match level {
                cli::ProverLevel::Base => UnrolledProverLevel::Base,
                cli::ProverLevel::RecursionUnrolled => UnrolledProverLevel::RecursionUnrolled,
                cli::ProverLevel::RecursionUnified => UnrolledProverLevel::RecursionUnified,
            };
            vk::generate_vk(&app_bin, &output, prover_level)?;
        }
        cli::Commands::VerifyProof { proof, vk, level } => {
            let prover_level = match level {
                cli::ProverLevel::Base => UnrolledProverLevel::Base,
                cli::ProverLevel::RecursionUnrolled => UnrolledProverLevel::RecursionUnrolled,
                cli::ProverLevel::RecursionUnified => UnrolledProverLevel::RecursionUnified,
            };
            vk::verify_proof(&proof, &vk, prover_level)?;
        }
    }

    Ok(())
}

fn init_tracing() -> Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init()
        .map_err(|e| anyhow::anyhow!("Can't initialize tracing subscriber: {e}"))?;
    Ok(())
}
