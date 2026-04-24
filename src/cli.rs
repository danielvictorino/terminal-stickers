use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::animation::TextEffect;
use crate::render::RendererChoice;

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(long, global = true, value_name = "DIR")]
    pub pack_dir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Animate(AnimateArgs),
    Chat(ChatArgs),
    List(ListArgs),
    Preview(PreviewArgs),
    Doctor,
    InitPack(InitPackArgs),
}

impl Default for Command {
    fn default() -> Self {
        Self::Chat(ChatArgs)
    }
}

#[derive(Debug, Clone, Args)]
pub struct ChatArgs;

#[derive(Debug, Clone, Args)]
pub struct AnimateArgs {
    #[arg(value_name = "TEXT", num_args = 1..)]
    pub text: Vec<String>,

    #[arg(short, long, value_enum, default_value = "typewriter")]
    pub effect: TextEffect,

    #[arg(long, default_value_t = 30)]
    pub fps: u64,

    #[arg(long, default_value_t = 36)]
    pub frames: usize,

    #[arg(long)]
    pub no_color: bool,
}

impl AnimateArgs {
    pub fn text(&self) -> String {
        self.text.join(" ")
    }
}

#[derive(Debug, Clone, Args)]
pub struct ListArgs {
    #[arg(short, long)]
    pub query: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct PreviewArgs {
    pub sticker: String,

    #[arg(short, long, default_value_t = 32)]
    pub width: u32,

    #[arg(long, value_enum, default_value = "auto")]
    pub renderer: RendererChoice,
}

#[derive(Debug, Clone, Args)]
pub struct InitPackArgs {
    pub name: String,
}
