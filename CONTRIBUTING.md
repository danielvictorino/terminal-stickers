# Contributing

Thanks for helping build Terminal Stickers.

## Development Setup

Install Rust stable, then run:

```sh
git clone https://github.com/danielvictorino/terminal-stickers.git
cd terminal-stickers
cargo run -- --pack-dir ./packs doctor
cargo run -- --pack-dir ./packs chat
```

Before opening a pull request:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

## Code Contributions

- Keep the CLI stable and scriptable.
- Keep sticker pack manifests backward compatible.
- Add tests for manifest parsing, pack discovery, renderer detection, and CLI behavior.
- Prefer small pull requests with one clear behavior change.

## Sticker Pack Contributions

Sticker packs must include:

- A `sticker-pack.toml` manifest.
- Image files in the pack folder.
- Clear license metadata.
- No copyrighted third-party images unless you have rights to contribute them.

Recommended asset shape:

- Static images: `512x512`, transparent PNG or WebP.
- Small file sizes.
- Descriptive IDs and tags.

## Documentation Contributions

Docs should be practical and tested against the current CLI. If a command or terminal behavior depends on a specific emulator, name that emulator and version when possible.
