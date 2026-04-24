# terminal-stickers

Terminal Stickers is a WhatsApp-style sticker experience for the terminal. It lets people keep local sticker packs, search by tag, preview small expressive images, and send them inside a terminal chat UI.

The project is built as an open-source Rust CLI/TUI and is designed around real terminal image capabilities: Kitty graphics, Sixel, iTerm2 inline images, and Unicode/ANSI fallbacks.

## Install

macOS and Linux:

```sh
curl -fsSL https://raw.githubusercontent.com/danielvictorino/terminal-stickers/main/install.sh | sh
```

Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/danielvictorino/terminal-stickers/main/install.ps1 | iex
```

The install scripts download the latest GitHub Release for your OS and architecture, verify the release checksum when available, place the `terminal-stickers` binary in a user-local bin directory, and install the bundled starter sticker packs beside it.

## Quickstart

```sh
terminal-stickers doctor
terminal-stickers list
terminal-stickers preview laugh
terminal-stickers animate "mandou bem" --effect glitch
terminal-stickers chat
```

Use a custom sticker pack directory:

```sh
terminal-stickers --pack-dir ./packs list
```

Create a new local sticker pack:

```sh
terminal-stickers --pack-dir ./packs init-pack "My Reactions"
```

## Sticker Packs

A pack is a folder with a `sticker-pack.toml` manifest and image files:

```toml
id = "starter"
name = "Starter Pack"
author = "terminal-stickers"
license = "CC0-1.0"

[[stickers]]
id = "laugh"
name = "Laugh"
file = "stickers/laugh.ppm"
tags = ["reaction", "funny"]
```

WhatsApp static stickers are commonly normalized to `512x512` WebP assets. Terminal Stickers accepts normal image formats supported by the Rust `image` crate, including PNG, JPEG, WebP, GIF, and PNM/PPM.

## Terminal Support

Terminal image support is fragmented, so the project uses progressive rendering:

1. Kitty graphics protocol when available.
2. Sixel-capable terminals, including modern Windows Terminal builds.
3. iTerm2 inline image protocol.
4. Unicode half-block rendering with ANSI truecolor.
5. Plain text placeholders.

The `preview` command currently uses the portable half-block renderer. The TUI and renderer modules are structured so native protocol rendering can be expanded without changing the sticker pack format.

## Animated Text Effects

Terminal Stickers includes a small native Rust animation layer inspired by terminal text effect tools. It is intentionally lightweight and does not require Python or external commands.

```sh
terminal-stickers animate "isso foi cinema" --effect typewriter
terminal-stickers animate "nao tankei" --effect wipe --fps 24
terminal-stickers animate "bugou tudo" --effect glitch --frames 48
terminal-stickers animate "mandou bem" --effect burst
```

Supported effects:

- `typewriter`: reveals text one character at a time.
- `wipe`: sweeps across a line using ASCII placeholder cells.
- `glitch`: scrambles unrevealed characters before resolving to the final text.
- `burst`: wraps the text in expanding ASCII reaction marks.

The chat TUI also uses this layer for a short send animation when a sticker is selected.

## Development

Requirements:

- Rust stable toolchain.
- Git.

Run locally:

```sh
cargo run -- doctor
cargo run -- --pack-dir ./packs list
cargo run -- --pack-dir ./packs preview laugh
cargo run -- animate "mandou bem" --effect glitch
cargo run -- --pack-dir ./packs chat
```

Quality checks:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for code, sticker pack, and documentation contribution guidelines.

## License

MIT. See [LICENSE](LICENSE).
