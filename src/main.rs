use clap::Parser;

fn main() -> anyhow::Result<()> {
    terminal_stickers::run(terminal_stickers::cli::Cli::parse())
}
