# simply-edit

> A simple CLI tool for manipulating images.

simply-edit is a conveniant command-line utility for everyday image tasks: flip, rotate, invert, grayscale, and convert between common formats like PNG, JPG, ICO, and SVG. It is designed to be easy use with sensible defaults, optional in-place replacement, and straightforward commands that help you process images quickly.

---

## Installation

### Prerequisites

- **Rust 1.56+** — Install from [rustup.rs](https://rustup.rs/)

### Install from Source

Install `simply-edit` to your PATH so you can run `simply` from anywhere:

```bash
git clone https://github.com/Maelkiz/simply-edit.git
cd simply-edit
cargo install --path .
```

The binary is installed to `~/.cargo/bin/`, which is usually already in your `$PATH`.

### Verify Installation

```bash
simply --help
```

---

## Quick Start

Run commands with:

```bash
simply <command> <args>
```

### Command Overview

| Command | What it does |
| --- | --- |
| `help` | Prints a detailed overview of the available commands |
| `flip` | Flip image (interactive by default, or via direction flags) |
| `rotate` | Rotate image (`90`, `180`, `270`) |
| `invert` | Invert image colors |
| `grayscale` | Convert image to grayscale |
| `convert` | Convert between PNG/JPG/ICO/SVG |

### Available Flags

**Common transform flag (`flip`, `rotate`, `invert`, `grayscale`):**

- `-r`, `--replace`: Replace the source file after a successful write.

**`flip` flags:**

- `--horizontal`: Flip horizontally without interactive prompt.
- `--vertical`: Flip vertically without interactive prompt.

**`convert` flags:**

- `-s`, `--scale <factor>`: Scale SVG rasterization output.
- `-w`, `--width <px>`: Set raster output width (for SVG input).
- `-h`, `--height <px>`: Set raster output height (for SVG input).

### Output File Type

The output file type is determined by the extension you provide in the output path (for example, `.png`, `.jpg`, `.ico`, or `.svg`).

### Format Support

- **PNG**: Full support, preserves transparency
- **JPG/JPEG**: Supported for input and output
- **ICO**: Supported for input and output. Images larger than 256×256 pixels are automatically resized while maintaining aspect ratio
- **SVG**: Supported as a `convert` output format via vector tracing (raster image -> SVG)
- **SVG input**: Supported for `convert` output to raster formats via `resvg` at 1.0 scale

---

## Contributing / Development Setup

This section is for contributors and local development.

### Clone the Repository

```bash
git clone https://github.com/Maelkiz/simply-edit.git
cd simply-edit
```

### Build

**Development build:**

```bash
cargo build
./target/debug/simply <command> <args>
```

**Release build:**

```bash
cargo build --release
./target/release/simply <command> <args>
```

### Run Without Installing

```bash
cargo run -- <command> <args>
```

### Run Tests

```bash
cargo test
```
