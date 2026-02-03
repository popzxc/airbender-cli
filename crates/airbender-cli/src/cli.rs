use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "airbender", version, about = "Airbender proving system CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Runs the binary with provided input via the simulator.
    Run {
        app_bin: PathBuf,
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        cycles: Option<usize>,
    },
    /// Runs the binary and emits a flamegraph SVG.
    Flamegraph {
        app_bin: PathBuf,
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long, default_value = "flamegraph.svg")]
        output: PathBuf,
        #[arg(short, long)]
        cycles: Option<usize>,
        /// Sampling rate: one sample per N cycles.
        #[arg(long, default_value_t = 100)]
        sampling_rate: usize,
        /// Generate inverse flamegraph.
        #[arg(long)]
        inverse: bool,
        /// Optional path to ELF symbols file.
        #[arg(long)]
        elf_path: Option<PathBuf>,
    },
    /// Generates a proof and writes it as bincode to the output file.
    Prove {
        app_bin: PathBuf,
        #[arg(short, long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
        /// Worker thread count for the unrolled prover.
        #[arg(long)]
        threads: Option<usize>,
    },
    /// Generates unified VKs for the recursion layer and writes a single bincode file.
    GenerateVk {
        app_bin: PathBuf,
        #[arg(short, long, default_value = "vk.bin")]
        output: PathBuf,
    },
    /// Verifies a proof against VKs.
    VerifyProof {
        proof: PathBuf,
        #[arg(long)]
        vk: PathBuf,
    },
}
