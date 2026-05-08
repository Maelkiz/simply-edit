# simply-edit

> A simple CLI tool for manipulating images.

simply-edit is a convenient command-line utility for everyday image tasks: flip, rotate, invert, grayscale, resize, and convert between common formats like PNG, JPG, ICO, SVG, and WebP. It is designed to be easy to use with sensible defaults, optional in-place replacement, and straightforward commands that help you process images quickly.

---

## Installation

### Prerequisites

- **Rust 1.85 or later** — Install from [rustup.rs](https://rustup.rs/)

### Install from Source

Install simply-edit so you can run the `simply` commands from anywhere:

```bash
git clone https://github.com/Maelkiz/simply-edit.git
cd simply-edit
cargo install --path .
```

### Verify Installation

```bash
command -v simply
simply --help
```

Cargo installs binaries to: 

```
$HOME/.cargo/bin
```

If `simply` is not found, add this directory to your PATH and refresh your shell.

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
| `convert` | Convert between PNG/JPG/ICO/WebP formats |
| `vectorize` | Convert a raster image to SVG |
| `rasterize` | Convert an SVG to a raster image |
| `info` | Display image metadata and properties |
| `view` | Display an image inline in the terminal (Kitty, WezTerm, or Ghostty) |

To get an more detailed description of any given command and its available flags run:

```bash
simply <command> --help
```

### Output Path

If you omit the output path, the tool generates one automatically: transforms keep the source format (e.g. `image.png` → `image_fliph.png`), while `vectorize` and `rasterize` switch to `.svg` and `.png` respectively. When you provide an explicit output path, the format is determined by its extension.

### Common Examples

#### Transforms

```bash
# Flip (interactive: choose horizontal or vertical)
simply flip ./image.png

# Flip bypassing interactive mode
simply flip --vertical ./image.png

# Rotate (interactive: choose 90, 180, or 270 degrees)
simply rotate ./image.png

# Rotate bypassing interactive mode
simply rotate --angle 90 ./image.png

# Replace original file in-place
simply rotate --angle 180 --replace ./image.png
```

#### Format Conversion

```bash
# Convert a raster image to SVG
simply vectorize ./image.png

# Convert an SVG to a raster image at 2× scale
simply rasterize -s 2 ./icon.svg ./icon.png
```

#### Batch Processing

```bash
# Invert all images in a directory
simply invert ./photos/

# Convert all JPGs to WebP, writing results to a separate directory
simply convert --format webp ./photos/ --output-dir ./converted/

# Grayscale only matching files, recursively
simply grayscale ./photos/ -R --pattern "^photo_"
```

#### View & Preview

```bash
# Display an image inline in the terminal
simply view ./photo.png

# Preview a transform without saving
simply flip --horizontal --preview ./photo.png
simply rotate --angle 90 --preview ./photo.png
simply grayscale --preview ./photo.png
simply vectorize --preview ./photo.png
```

### Format Support

- **PNG**: Full support, preserves transparency
- **JPG/JPEG**: Supported for input and output
- **ICO**: Supported for input and output. Images larger than 256×256 pixels are automatically resized while maintaining aspect ratio
- **WebP**: Supported for input and output
- **SVG output**: Raster images can be vectorized to SVG via `vectorize` (or `convert` with an `.svg` destination)
- **SVG input**: SVG files can be rasterized via `rasterize` (supports `--scale`, `--width`, `--height`) or `convert` (at native resolution)

---