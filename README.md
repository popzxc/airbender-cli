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

## Running

While you can run the tool via `cargo run --bin airbender-cli --release`, it is recommended to either install it via
`cargo install` or access the build version in `./target/release/airbender-cli`.

Usage follows the default flow:

- `airbender-cli <command> [options]`

You can run the binary via Cargo:

```sh
# Run the simulator
./target/release/airbender-cli run ./path/to/app.bin --input ./input.hex

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

## Caveats / Important Notes

- Input files are hex strings representing 32-bit words. Whitespace is ignored and an optional `0x` prefix is allowed. The length must be a multiple of 8 hex characters.
- Proofs and VKs are written as bincode payloads; consumers must use compatible versions.

## License

Licensed under MIT or Apache-2.0, at your option.
