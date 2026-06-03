# simply-edit

> A simple CLI tool for manipulating images.

simply-edit is a convenient command-line utility for everyday image tasks: flip, flop, rotate, invert, grayscale, binarize, pad, resize, and convert between common formats. It is designed to be easy to use, with sensible defaults, straightforward commands, and quality-of-life features, such as optional in-place replacement, batch operations, and view/preview functionality using the [Kitty Terminal Graphics Protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol).

---

## Installation

### Prerequisites

- **Rust 1.85 or later** — Install from [rustup.rs](https://rustup.rs/)

### Install from Source

Install simply-edit so you can run the `simply` commands from anywhere:

```bash
cargo install --git https://github.com/Maelkiz/simply-edit.git
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

So, if `simply` is not found, add this directory to your PATH and refresh your shell.

---

## Quick Start

Run commands with:

```bash
simply <command> <args>
```

### Command Overview

| Command | What it does |
| --- | --- |
| `help` | Prints an overview of the available commands |
| `flip` | Mirror an image vertically (top to bottom) |
| `flop` | Mirror an image horizontally (left to right) |
| `rotate` | Rotate image (interactive by default, or explicit `90`/`180`/`270`) |
| `invert` | Invert image colors |
| `grayscale` | Convert image to grayscale |
| `binarize` | Convert image to pure black and white at a brightness cutoff |
| `pad` | Add padding (transparent or colored) around an image |
| `resize` | Resize an image to specified dimensions |
| `convert` | Convert between PNG/JPG/ICO/WebP formats |
| `vectorize` | Convert a raster image to SVG |
| `rasterize` | Convert an SVG to a raster image |
| `info` | Display image metadata and properties |
| `view` | Display an image inline in the terminal (Kitty, WezTerm, or Ghostty) |

To get a more detailed description of any given command and its available flags, run:

```bash
simply <command> --help
```

### Output Path

If you omit the output path, the tool generates one automatically: transforms keep the source format (e.g., `image.png` → `image_flipv.png`), while `vectorize` and `rasterize` switch to `.svg` and `.png` respectively. When you provide an explicit output path, the format is determined by its extension.

### Common Examples

#### Transforms

```bash
# Flip (mirror vertically)
simply flip ./image.png

# Flop (mirror horizontally)
simply flop ./image.png

# Rotate (interactive: choose 90, 180, or 270 degrees)
simply rotate ./image.png

# Rotate bypassing interactive mode
simply rotate --angle 90 ./image.png

# Replace original file in-place
simply rotate --angle 180 --replace ./image.png

# Binarize with default threshold (128)
simply binarize ./image.png

# Binarize with custom threshold
simply binarize --threshold 200 ./image.png

# Pad with 20px on every side (default when no size flags given)
simply pad ./image.png

# Pad individual sides
simply pad --top 10 --bottom 10 ./image.png

# Pad left and right equally using the -x shorthand
simply pad -x 20 ./image.png

# Pad top and bottom equally using the -y shorthand
simply pad -y 15 ./image.png

# Pad with a custom fill color (hex: rrggbb or rrggbbaa)
simply pad --top 20 -x 15 --color 00ff00 ./image.png

# Pad and replace the original file in-place
simply pad --top 5 --replace ./image.png
```

#### Format Conversion

```bash
# Convert between formats
simply convert ./photo.png ./photo.jpg

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

# Binarize all images in a directory with a custom threshold
simply binarize --threshold 100 ./scans/ --output-dir ./cleaned/
```

#### View & Preview

```bash
# Display an image inline in the terminal
simply view ./photo.png

# Preview a transform without saving
simply flip --preview ./photo.png
simply flop --preview ./photo.png
simply rotate --angle 90 --preview ./photo.png
simply grayscale --preview ./photo.png
simply binarize --preview ./photo.png
simply pad --top 20 -x 10 --preview ./photo.png
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
