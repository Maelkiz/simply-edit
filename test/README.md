## Manual Testing

This folder is for manual testing during development. You can of course test however you like, but I recommend:

- Use the /test/output directory as destination path when testing manually.
- Use /test/test.png for manual tests inless testing -r/--replace
- The replace flags can be tested on images outputed by previous tests

Example:
```
cargo run -- invert test/test.png test/output/inverted.jpg

cargo run -- flipv -r test/output/inverted.jpg
```