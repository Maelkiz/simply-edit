# simple-edit

> A simple image CLI tool for flipping and rotating images.

## Usage

### Flip Horizontal

Flip an image horizontally (left-right mirror).

```bash
simple-edit fliph <path-to-image> [output-path]
```

- `<path-to-image>`: Path to the source image file
- `[output-path]` (optional): Path for the output file. If omitted, saves as `{filename}_fliph.{ext}` in the same directory

**Examples:**
```bash
cargo run -- fliph image.jpg                    # Saves to image_fliph.jpg
cargo run -- fliph image.jpg output.png         # Saves to output.png (converts to PNG)
```

### Flip Vertical

Flip an image vertically (top-bottom mirror).

```bash
simple-edit flipv <path-to-image> [output-path]
```

- `<path-to-image>`: Path to the source image file
- `[output-path]` (optional): Path for the output file. If omitted, saves as `{filename}_flipv.{ext}` in the same directory

**Examples:**
```bash
cargo run -- flipv image.jpg                    # Saves to image_flipv.jpg
cargo run -- flipv image.png output.ico         # Saves to output.ico (converts to ICO, auto-resized to 256x256)
```

### Rotate

Rotate an image by 90, 180, or 270 degrees.

```bash
simple-edit rotate <degrees> <path-to-image> [output-path]
```

- `<degrees>`: Rotation angle in degrees (90, 180, or 270)
- `<path-to-image>`: Path to the source image file
- `[output-path]` (optional): Path for the output file. If omitted, saves as `{filename}_rotate{degrees}.{ext}` in the same directory

**Examples:**
```bash
cargo run -- rotate 90 image.jpg                # Saves to image_rotate90.jpg
cargo run -- rotate 180 image.png output.jpg    # Saves to output.jpg (converts to JPG)
cargo run -- rotate 270 image.jpg output.ico    # Saves to output.ico (converts to ICO, auto-resized)
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
