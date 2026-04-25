# Playground

This directory is for manual testing and experimentation.

## Testing Installed Binary

You can run the following command from the project root to install a release binary to your PATH:
```bash
cargo install --path .
```

After installation completes you can test out the `simply` commands like so:
```bash
simply <command> <args>
```

Run this for an overview of the available commands:
```bash
simply help
```

Example:
```bash
cd playground

simply rotate 90 test.png

simply convert test.png vectorized.svg
```

## Testing During Development

While developing you can test by passing the commands and arguments directly to `cargo run` like so:
```bash
cargo run -- <command> <args>
```

Run this for an overview of the available commands:
```bash
cargo run -- help
```

Example:
```bash
# Inverts the colors of test.png and outputs it as a jpg
cargo run -- invert playground/test.png playground/inverted.jpg

# Outputs a flipped version of inverted.jpg replacing the original
cargo run -- flip -r playground/inverted.jpg
```