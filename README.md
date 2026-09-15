# simply-edit

> A simple CLI tool for manipulating images.

simply-edit is a convenient command-line utility for everyday image tasks: flip, rotate, invert, grayscale, binarize, pad, resize, scale, stretch, and convert between common formats. It is designed to be easy to use, with sensible defaults, straightforward commands, and quality-of-life features, such as optional in-place replacement, batch operations, and view/preview functionality using the [Kitty Terminal Graphics Protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol).

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
| `help` | Prints an overview of the available commands (equivalent to `simply --help`) |
| `flip` | Mirror an image horizontally (`--horizontal`), vertically (`--vertical`), both, or run it interactively by giving no flags |
| `rotate` | Rotate image (interactive by default, or explicit `90`/`180`/`270`) |
| `invert` | Invert image colors |
| `grayscale` | Convert image to grayscale |
| `binarize` | Convert image to pure black and white at a brightness cutoff |
| `pad` | Add padding (transparent or colored) around an image |
| `resize` | Resize an image to specified dimensions |
| `scale` | Scale an image uniformly by a factor (`--factor`) |
| `stretch` | Stretch an image independently along each axis (`--horizontal`, `--vertical`) |
| `convert` | Convert between PNG/JPG/ICO/WebP formats |
| `vectorize` | Convert a raster image to SVG (auto-downscales large images for speed; use `--full-quality` to disable) |
| `rasterize` | Convert an SVG to a raster image |
| `info` | Display image metadata and properties |
| `view` | Display an image inline in the terminal (requires Kitty graphics protocol support (Kitty, WezTerm, or Ghostty)) |

To get a more detailed description of any given command and its available flags, run:

```bash
simply <command> --help
```

### Interactive Commands

For those who do not want to memorize a bunch of flags, you can run all commands with an image path as the only argument:

```bash
simply <command> <path-to-img>
```

For some commands this will start an interactive prompt, while others run with a sane default. Use `simply <command> --help` for information on a specific command's behaviour.

### Shorthands

Some frequently used commands have convenient shorthands:

```bash
# Concisely flip an image horizontally or vertically
simply fliph ./photo.png # Equivalent to `simply flip --horizontal ./photo.png`
simply flipv ./photo.png # Equivalent to `simply flip --vertical ./photo.png`

# Concisely rotate an image at a specific angle
simply rotate90 ./image.png
simply rotate180 ./image.png
simply rotate270 ./image.png

# Shorthands take the same flags as the command they stand for
simply rotate180 --replace ./image.png # Rotates in place, overwriting the source file
```

### Format Conversion

For explicit format conversions the following commands are available:

```bash
# Convert between formats
simply convert ./photo.png ./photo.jpg

# Convert a raster image to SVG
simply vectorize ./image.png

# Convert an SVG to a raster image
simply rasterize ./icon.svg
```

Implicit conversions are also supported. When you provide an explicit output path, its file extension determines the output format:

```bash
# Implicit conversion between formats
simply <command> ./photo.png ./photo.jpg
```

### Batch Processing

Batch image processing is supported and is used like this:

```bash
# Invert all images in a directory
simply invert ./photos/

# Convert all images in the directory to WebP, writing results to a separate directory
simply convert --format webp ./photos/ --output-dir ./converted/

# Grayscale only matching files, recursively
simply grayscale ./photos/ -r --pattern "^photo_"

# Binarize all images in a directory with a custom threshold
simply binarize --threshold 100 ./scans/ --output-dir ./cleaned/
```

### View & Preview

If your terminal supports the [Kitty Terminal Graphics Protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol) you can display images and preview the results of commands:

```bash
# Display an image inline in the terminal
simply view ./photo.png

# Preview a transform without saving
simply <command> --preview ./image.png
```

### Format Support

- **PNG**: Full support, preserves transparency
- **JPG/JPEG**: Supported for input and output
- **ICO**: Supported for input and output. Images larger than 256×256 pixels are automatically resized while maintaining aspect ratio
- **WebP**: Supported for input and output
- **SVG output**: Raster images can be vectorized to SVG via `vectorize` (or `convert` with an `.svg` destination). Images larger than 2000px on the long edge are automatically downscaled before vectorization for performance, with the SVG retaining the original dimensions via a `viewBox`. Pass `--full-quality` to vectorize at full resolution.
- **SVG input**: SVG files can be rasterized via `rasterize` (supports `--scale`, `--width`, `--height`) or `convert` (at native resolution)

### Output Path

If you omit the output path, the tool generates one automatically: transforms keep the source format (e.g., `image.png` → `image_flipv.png`, `image_fliph.png`, or `image_flipxy.png` for flip), while `vectorize` and `rasterize` switch to `.svg` and `.png` respectively.

---