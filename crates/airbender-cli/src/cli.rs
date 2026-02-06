use clap::{Parser, Subcommand, ValueEnum};
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
    /// Runs the binary via the transpiler JIT.
    RunTranspiler {
        app_bin: PathBuf,
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        cycles: Option<usize>,
        /// Optional path to the .text section (raw instructions).
        #[arg(long)]
        text_path: Option<PathBuf>,
    },
    /// Generates a proof and writes it as bincode to the output file.
    Prove {
        app_bin: PathBuf,
        #[arg(short, long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
        /// Prover backend to use.
        #[arg(long, value_enum, default_value_t = ProverBackend::Gpu)]
        backend: ProverBackend,
        /// Worker thread count for the unrolled prover.
        #[arg(long, short)]
        threads: Option<usize>,
        /// Cycle bound for CPU proving.
        #[arg(long)]
        cycles: Option<usize>,
        /// RAM bound in bytes for CPU proving.
        #[arg(long)]
        ram_bound: Option<usize>,
        /// Max prover level to generate.
        #[arg(long, value_enum, default_value_t = ProverLevel::RecursionUnified)]
        level: ProverLevel,
    },
    /// Generates VKs for the requested level and writes a single bincode file.
    GenerateVk {
        app_bin: PathBuf,
        #[arg(short, long, default_value = "vk.bin")]
        output: PathBuf,
        /// Max prover level to generate.
        #[arg(long, value_enum, default_value_t = ProverLevel::RecursionUnified)]
        level: ProverLevel,
    },
    /// Verifies a proof against VKs.
    VerifyProof {
        proof: PathBuf,
        #[arg(long)]
        vk: PathBuf,
        /// Proof level to verify.
        #[arg(long, value_enum, default_value_t = ProverLevel::RecursionUnified)]
        level: ProverLevel,
    },
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum ProverLevel {
    Base,
    RecursionUnrolled,
    RecursionUnified,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum ProverBackend {
    Cpu,
    Gpu,
}
