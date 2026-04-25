# simply-edit

> A simple CLI tool for manipulating images.

simply-edit is a convenient command-line utility for everyday image tasks: flip, rotate, invert, grayscale, and convert between common formats like PNG, JPG, ICO, SVG, and WebP. It is designed to be easy to use with sensible defaults, optional in-place replacement, and straightforward commands that help you process images quickly.

---

## Installation

### Prerequisites

- **Rust (stable, edition 2024 compatible)** — Install from [rustup.rs](https://rustup.rs/)

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
| `rotate` | Rotate image (interactive by default, or explicit `90`/`180`/`270`) |
| `invert` | Invert image colors |
| `grayscale` | Convert image to grayscale |
| `convert` | Convert between PNG/JPG/ICO/SVG/WebP |
| `vectorize` | Convert a raster image to SVG |
| `rasterize` | Convert an SVG to a raster image |

### Available Flags

**Common transform flag (`flip`, `rotate`, `invert`, `grayscale`):**

- `-r`, `--replace`: Replace the source file after a successful write.

**`flip` flags:**

- `--horizontal`: Flip horizontally without interactive prompt.
- `--vertical`: Flip vertically without interactive prompt.

**`rotate` flags:**

- `--angle <90|180|270>`: Rotation angle (interactive prompt if omitted).

**`vectorize` flags:**

- `--fast`: Faster conversion with lower fidelity.

**`rasterize` flags:**

- `-s`, `--scale <factor>`: Scale factor for rasterization.
- `-w`, `--width <px>`: Output width in pixels.
- `-H`, `--height <px>`: Output height in pixels.

**Batch flags (available on all commands):**

- `--pattern <regex>`: Regex pattern to filter filenames.
- `--output-dir <path>`: Output directory for batch results.
- `-R`, `--recursive`: Process subdirectories recursively.
- `--format <fmt>`: Output format for batch convert (e.g. `png`, `jpg`, `webp`).

### Output Path

If you omit the output path, the tool generates one automatically based on the command — for example, `image.png` becomes `image_fliph.png` after a horizontal flip. The output file type is determined by the extension you provide (for example, `.png`, `.jpg`, `.ico`, `.svg`, or `.webp`).

### Common Examples

```bash
# Flip (interactive: choose horizontal or vertical)
simply flip ./image.png

# Flip bypassing interactive mode
simply flip --vertical ./image.png

# Rotate (interactive: choose 90, 180, or 270 degrees)
simply rotate ./image.png

# Rotate bypassing interactive mode
simply rotate --angle 90 ./image.png

# Replace original file after transform
simply rotate --angle 180 --replace ./image.png

# Convert raster image to SVG
simply vectorize ./image.png

# Convert SVG to raster image with scale
simply rasterize -s 2 ./icon.svg ./icon.png

# Batch: invert all images in a directory
simply invert ./photos/

# Batch: convert all JPGs to WebP with output directory
simply convert --format webp ./photos/ --output-dir ./converted/

# Batch: process only matching files recursively
simply grayscale ./photos/ -R --pattern "^photo_"
```

### Format Support

- **PNG**: Full support, preserves transparency
- **JPG/JPEG**: Supported for input and output
- **ICO**: Supported for input and output. Images larger than 256×256 pixels are automatically resized while maintaining aspect ratio
- **WebP**: Supported for input and output
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
