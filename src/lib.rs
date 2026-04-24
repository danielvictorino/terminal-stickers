pub mod animation;
pub mod cli;
pub mod manifest;
pub mod packs;
pub mod paths;
pub mod render;
pub mod tui;

use anyhow::Result;

pub fn run(cli: cli::Cli) -> Result<()> {
    let pack_dir = paths::resolve_pack_dir(cli.pack_dir.as_deref());

    match cli.command.unwrap_or_default() {
        cli::Command::Animate(args) => animation::play_text(
            &args.text(),
            args.effect,
            args.fps,
            args.frames,
            args.no_color,
        ),
        cli::Command::Chat(args) => tui::run_chat(&pack_dir, args),
        cli::Command::List(args) => packs::print_list(&pack_dir, args),
        cli::Command::Preview(args) => {
            let packs = packs::discover_packs(&pack_dir)?;
            let sticker = packs::resolve_sticker(&args.sticker, &packs)?;
            println!("{} -> {}", sticker.qualified_id(), sticker.path.display());
            render::print_preview(&sticker.path, args.width, args.renderer)
        }
        cli::Command::Doctor => {
            render::print_doctor(&pack_dir);
            Ok(())
        }
        cli::Command::InitPack(args) => packs::init_pack(&pack_dir, &args.name),
    }
}
