# Airbender CLI

## Introduction

Airbender CLI is a command-line interface for the Airbender proving system.
It helps you run the RISC-V simulator, generate proofs, manage verification keys, and profile execution locally
so you can iterate on proving workflows without wiring up a larger stack.

> [!WARNING]  
> Under active development, no stability guarantees whatsoever. Use at your own risk.

## Building

```sh
RUST_MIN_STACK=16777216 cargo build --release
```

This project does always have GPU prover enabled, so in case you don't have CUDA set up locally, you can still compile it by setting:

```
ZKSYNC_USE_CUDA_STUBS=true
```

## Running

While you can run the tool via `cargo run --bin airbender-cli --release`, it is recommended to either install it via
`cargo install` or access the build version in `./target/release/airbender-cli`.

Usage follows the default flow:

- `airbender-cli <command> [options]`

You can run the binary via Cargo:

```sh
# Run the simulator
./target/release/airbender-cli run ./path/to/app.bin --input ./input.hex

# Run via `riscv_transpiler` JIT
./target/release/airbender-cli run-transpiler ./path/to/app.bin --input ./input.hex

# Generate a flamegraph SVG
./target/release/airbender-cli flamegraph ./path/to/app.bin --input ./input.hex --output flamegraph.svg

# Generate a proof
./target/release/airbender-cli prove ./path/to/app.bin --input ./input.hex --output proof.bin

# Generate unified VKs
./target/release/airbender-cli generate-vk ./path/to/app.bin --output vk.bin

# Verify a proof
./target/release/airbender-cli verify-proof ./proof.bin --vk ./vk.bin
```

Use `--help` for the full reference and the complete set of options.

## Debugging circuits

Proving supports multiple levels through `--level` (`base`, `recursion-unrolled`, `recursion-unified`).
When debugging circuit issues, start from the base layer:

```sh
# Generate a base-layer proof
./target/release/airbender-cli prove <app.bin> -i <encoded_input.txt> --level base --output <proof.bin>

# Generate base-layer VKs
./target/release/airbender-cli generate-vk <app.bin> --level base --output <vk.bin>

# Verify the base-layer proof
./target/release/airbender-cli verify-proof <proof.bin> --vk <vk.bin> --level base
```

If proving fails and you need more insight into unsatisfied constraints, generate a CPU proof.
Build with debug circuit flags first:

```sh
RUST_MIN_STACK=16777216 cargo build --release --features debug_circuits
```

Then run base-layer proving on CPU:

```sh
./target/release/airbender-cli prove <app.bin> -i <encoded_input.txt> --backend cpu --level base --cycles <cycles> --ram-bound <ram_bound_bytes> --output <proof.bin>
```

`--cycles` is optional. If omitted, the CLI estimates it automatically by running the program first via the transpiler.

## Caveats / Important Notes

- Input files are hex strings representing 32-bit words. Whitespace is ignored and an optional `0x` prefix is allowed. The length must be a multiple of 8 hex characters.
- Proofs and VKs are written as bincode payloads; consumers must use compatible versions.

## License

Licensed under MIT or Apache-2.0, at your option.
