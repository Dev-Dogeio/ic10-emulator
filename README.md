# IC10 Emulator

## Overview

The IC10 Emulator is a Rust-based project designed to emulate the behavior of Stationeers IC10 programmable chips.

## Features

-   IC10 instruction set support\*
-   Parsing and execution of IC10 programs.
-   Cable network simulation.
-   Peripheral device emulation (eg sensors/memory chips, more devices will be implemented).
-   Unit tests for core functionality.
-   Modular design for easy extension.

\* Not yet complete, but the vast majority of instructions/IC10 features are supported

## Getting Started

### Prerequisites

-   Rust (latest stable version). You can install Rust using [rustup](https://rustup.rs/).

### Building the Project

1. Clone the repository:
    ```bash
    git clone https://github.com/Dev-Dogeio/ic10-emulator.git
    cd IC10-emulator
    ```
2. Build the project:
    ```bash
    cargo build
    ```

### Running the Emulator

To run the emulator, use the following command:

```bash
cargo run
```

### Running Tests

To execute the unit tests:

```bash
cargo test
```

## Contributing

Contributions are welcome! If you have ideas for improvements or find any issues, feel free to open an issue or submit a pull request.
