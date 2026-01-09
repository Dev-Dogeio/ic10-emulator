# IC10 Emulator

## Overview

The IC10 Emulator is a Rust-based project that emulates the Stationeers IC10 programmable chip and simulates device networks and atmospherics for testing and experimentation.

## Features

-   IC10 instruction set parsing and execution (not yet entirely complete)
-   Cable network simulation (`CableNetwork`) with batch read/write modes
-   Atmospheric simulation subsystem (`atmospherics`) with:
    -   Gas types, moles, and energy tracking (`GasType`, `Mole`, `GasMixture`)
    -   Ideal gas helpers and constants
    -   `AtmosphericNetwork` for shared gas mixtures between devices
-   Devices API and examples:
    -   `Filtration`, `DaylightSensor`, `LogicMemory`, `ICHousing` devices
-   Modular design for easy extension and testing

---

## Getting Started

### Prerequisites

-   Rust (latest stable). Install via [rustup](https://rustup.rs/).

### Building

```bash
git clone https://github.com/Dev-Dogeio/ic10-emulator.git
cd IC10-emulator
cargo build
```

### Building WASM

Prerequisites:

-   Install the `wasm-bindgen` CLI: `cargo install wasm-bindgen-cli`

Usage:

-   `cargo xtask build-wasm [--release]`
-   The generated js bindings will be placed into the `app/pkg/` directory.

### Running the example

The example in `src/main.rs` includes a short Filtration device demo and an IC program test. Run it with:

```bash
cargo run
```

### Running Tests

To execute the unit tests:

```bash
cargo test
```

## Contributing

Contributions welcome! Open issues or PRs. Please run `cargo test` and `cargo clippy`/`cargo fmt` before submitting changes.

---
