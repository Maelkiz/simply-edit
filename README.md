# simple-edit

> A simple CLI tool for manipulating images.

## Installation & Setup

### Prerequisites

- **Rust 1.56+** — Install from [rustup.rs](https://rustup.rs/)

### Clone the Repository

```bash
git clone https://github.com/yourusername/simple-edit.git
cd simple-edit
```

### Build from Source

**Development build:**
```bash
cargo build
./target/debug/simple-edit <command> <args>
```

**Release build (optimized):**
```bash
cargo build --release
./target/release/simple-edit <command> <args>
```

### Run Without Building

You can run commands directly from the repository without building:

```bash
cargo run -- <command> <args>
```

### Run Tests

Execute the unit test suite:

```bash
cargo test
```

## Usage

### Flip Horizontal

Flip an image horizontally (left-right mirror).

```bash
simple-edit fliph [-r|--replace] <path-to-image> [output-path]
```

- `<path-to-image>`: Path to the source image file
- `-r` / `--replace`: Replace the source file in place after the transformed image is written successfully
- `[output-path]` (optional): Path for the output file. Omit this when using `-r` / `--replace`. If omitted, saves as `{filename}_fliph.{ext}` in the same directory

**Examples:**
```bash
cargo run -- fliph image.jpg                    # Saves to image_fliph.jpg
cargo run -- fliph image.jpg output.png         # Saves to output.png (converts to PNG)
cargo run -- fliph -r image.jpg                 # Replaces image.jpg after writing successfully
```

### Flip Vertical

Flip an image vertically (top-bottom mirror).

```bash
simple-edit flipv [-r|--replace] <path-to-image> [output-path]
```

- `<path-to-image>`: Path to the source image file
- `-r` / `--replace`: Replace the source file in place after the transformed image is written successfully
- `[output-path]` (optional): Path for the output file. Omit this when using `-r` / `--replace`. If omitted, saves as `{filename}_flipv.{ext}` in the same directory

**Examples:**
```bash
cargo run -- flipv image.jpg                    # Saves to image_flipv.jpg
cargo run -- flipv image.png output.ico         # Saves to output.ico (converts to ICO, auto-resized to 256x256)
cargo run -- flipv --replace image.jpg          # Replaces image.jpg after writing successfully
```

### Rotate

Rotate an image by 90, 180, or 270 degrees.

```bash
simple-edit rotate <degrees> [-r|--replace] <path-to-image> [output-path]
```

- `<degrees>`: Rotation angle in degrees (90, 180, or 270)
- `<path-to-image>`: Path to the source image file
- `-r` / `--replace`: Replace the source file in place after the transformed image is written successfully
- `[output-path]` (optional): Path for the output file. Omit this when using `-r` / `--replace`. If omitted, saves as `{filename}_rotate{degrees}.{ext}` in the same directory

**Examples:**
```bash
cargo run -- rotate 90 image.jpg                # Saves to image_rotate90.jpg
cargo run -- rotate 180 image.png output.jpg    # Saves to output.jpg (converts to JPG)
cargo run -- rotate 270 image.jpg output.ico    # Saves to output.ico (converts to ICO, auto-resized)
cargo run -- rotate 90 --replace image.jpg      # Replaces image.jpg after writing successfully
```

### Invert

Invert the colors in an image.

```bash
simple-edit invert [-r|--replace] <path-to-image> [output-path]
```

- `<path-to-image>`: Path to the source image file
- `-r` / `--replace`: Replace the source file in place after the inverted image is written successfully
- `[output-path]` (optional): Path for the output file. Omit this when using `-r` / `--replace`. If omitted, saves as `{filename}_invert.{ext}` in the same directory

**Examples:**
```bash
cargo run -- invert image.jpg                   # Saves to image_invert.jpg
cargo run -- invert image.jpg output.png        # Saves to output.png
cargo run -- invert -r image.jpg                # Replaces image.jpg after writing successfully
```

### Grayscale

Convert an image to grayscale.

```bash
simple-edit grayscale [-r|--replace] <path-to-image> [output-path]
```

- `<path-to-image>`: Path to the source image file
- `-r` / `--replace`: Replace the source file in place after the grayscale image is written successfully
- `[output-path]` (optional): Path for the output file. Omit this when using `-r` / `--replace`. If omitted, saves as `{filename}_grayscale.{ext}` in the same directory

**Examples:**
```bash
cargo run -- grayscale image.jpg               # Saves to image_grayscale.jpg
cargo run -- grayscale image.jpg output.png    # Saves to output.png
cargo run -- grayscale -r image.jpg            # Replaces image.jpg after writing successfully
```

### Convert

Convert between supported image formats (PNG, JPG, ICO).

```bash
simple-edit convert <path-to-image> <new-path>
```

- `<path-to-image>`: Path to the source image file
- `<new-path>`: Path and filename for the output. The file extension determines the output format

**Examples:**
```bash
cargo run -- convert image.png image.jpg        # Converts PNG to JPG
cargo run -- convert image.jpg image.ico        # Converts JPG to ICO (auto-resized to 256x256 if needed)
cargo run -- convert image.ico image.png        # Converts ICO to PNG
```

## Format Support

- **PNG**: Full support, preserves transparency
- **JPG/JPEG**: Supported for input and output
- **ICO**: Supported for input and output. Images larger than 256×256 pixels are automatically resized while maintaining aspect ratio

## Building

```bash
cargo build --release
./target/release/simple-edit <command> <args>
```

## Running

```bash
cargo run -- <command> <args>
```
