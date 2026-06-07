# master-console

A Rust-based Gameboy emulator.

## How to run

### Prerequisites

To run this project you need to install Rust.

On Linux:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

On Windows go to the official [Rust website](https://rust-lang.org/tools/install/).

To verify the installation:

```bash
rustc --version
cargo --version
```

### Run

To run the project simply build it:

```bash
make build
```

or

```bash
make release
```

and run it:

```bash
make run args="-p path/to/rom.gb"
```

or

```bash
cargo run <--release> -p path/to/rom.gb
```

## Notes

As of now the emulator is capable of running a game but it's missing major components such as audio emulation, save files, and other features.
