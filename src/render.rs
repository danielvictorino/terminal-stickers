use std::{env, path::Path};

use anyhow::{Context, Result};
use clap::ValueEnum;
use image::{imageops::FilterType, GenericImageView, ImageReader, Rgba};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RendererChoice {
    Auto,
    Kitty,
    Sixel,
    Iterm2,
    Halfblocks,
    Plain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackend {
    Kitty,
    Sixel,
    Iterm2,
    Halfblocks,
    Plain,
}

impl RenderBackend {
    pub fn name(self) -> &'static str {
        match self {
            Self::Kitty => "kitty",
            Self::Sixel => "sixel",
            Self::Iterm2 => "iterm2",
            Self::Halfblocks => "halfblocks",
            Self::Plain => "plain",
        }
    }
}

pub fn print_doctor(pack_dir: &Path) {
    let backend = choose_backend(RendererChoice::Auto);

    println!("terminal-stickers doctor");
    println!("pack dir: {}", pack_dir.display());
    println!("detected renderer: {}", backend.name());
    println!("TERM: {}", env::var("TERM").unwrap_or_else(|_| "unknown".to_string()));
    println!(
        "TERM_PROGRAM: {}",
        env::var("TERM_PROGRAM").unwrap_or_else(|_| "unknown".to_string())
    );

    if !pack_dir.exists() {
        println!("pack dir does not exist yet");
    }
}

pub fn print_preview(path: &Path, width: u32, choice: RendererChoice) -> Result<()> {
    let backend = choose_backend(choice);

    match backend {
        RenderBackend::Plain => {
            println!("[sticker: {}]", path.display());
            Ok(())
        }
        RenderBackend::Kitty
        | RenderBackend::Sixel
        | RenderBackend::Iterm2
        | RenderBackend::Halfblocks => {
            let rendered = render_halfblocks(path, width)?;
            print!("{rendered}");
            Ok(())
        }
    }
}

pub fn choose_backend(choice: RendererChoice) -> RenderBackend {
    match choice {
        RendererChoice::Auto => detect_from_env(),
        RendererChoice::Kitty => RenderBackend::Kitty,
        RendererChoice::Sixel => RenderBackend::Sixel,
        RendererChoice::Iterm2 => RenderBackend::Iterm2,
        RendererChoice::Halfblocks => RenderBackend::Halfblocks,
        RendererChoice::Plain => RenderBackend::Plain,
    }
}

pub fn detect_from_env() -> RenderBackend {
    let vars = env::vars().collect::<Vec<_>>();
    let pairs = vars.iter().map(|(key, value)| (key.as_str(), value.as_str()));
    detect_from_pairs(pairs)
}

pub fn detect_from_pairs<'a, I>(pairs: I) -> RenderBackend
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let vars = pairs
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect::<Vec<_>>();

    let get = |name: &str| {
        vars.iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    };

    let term = get("TERM").unwrap_or_default().to_lowercase();
    let term_program = get("TERM_PROGRAM").unwrap_or_default();
    let colorterm = get("COLORTERM").unwrap_or_default().to_lowercase();

    if get("KITTY_WINDOW_ID").is_some() || term.contains("kitty") {
        return RenderBackend::Kitty;
    }

    if get("WEZTERM_EXECUTABLE").is_some() || term_program.eq_ignore_ascii_case("WezTerm") {
        return RenderBackend::Iterm2;
    }

    if term_program.eq_ignore_ascii_case("iTerm.app") {
        return RenderBackend::Iterm2;
    }

    if get("WT_SESSION").is_some()
        || term.contains("foot")
        || term.contains("mlterm")
        || term.contains("sixel")
    {
        return RenderBackend::Sixel;
    }

    if colorterm.contains("truecolor") || colorterm.contains("24bit") {
        return RenderBackend::Halfblocks;
    }

    RenderBackend::Plain
}

pub fn render_halfblocks(path: &Path, width: u32) -> Result<String> {
    let image = ImageReader::open(path)
        .with_context(|| format!("failed to open {}", path.display()))?
        .with_guessed_format()
        .with_context(|| format!("failed to guess image format for {}", path.display()))?
        .decode()
        .with_context(|| format!("failed to decode {}", path.display()))?;

    let (source_width, source_height) = image.dimensions();
    let target_width = width.clamp(4, 120);
    let target_rows = ((source_height as f32 / source_width as f32) * target_width as f32 / 2.0)
        .ceil()
        .max(1.0) as u32;
    let target_height = target_rows * 2;
    let resized = image
        .resize_exact(target_width, target_height, FilterType::Triangle)
        .to_rgba8();

    let mut out = String::new();

    for y in (0..target_height).step_by(2) {
        for x in 0..target_width {
            let top = *resized.get_pixel(x, y);
            let bottom = *resized.get_pixel(x, y + 1);
            push_halfblock_cell(&mut out, top, bottom);
        }
        out.push_str("\x1b[0m\n");
    }

    Ok(out)
}

fn push_halfblock_cell(out: &mut String, top: Rgba<u8>, bottom: Rgba<u8>) {
    match (visible_rgb(top), visible_rgb(bottom)) {
        (Some([tr, tg, tb]), Some([br, bg, bb])) => out.push_str(&format!(
            "\x1b[38;2;{tr};{tg};{tb}m\x1b[48;2;{br};{bg};{bb}m▀"
        )),
        (Some([r, g, b]), None) => out.push_str(&format!("\x1b[38;2;{r};{g};{b}m▀")),
        (None, Some([r, g, b])) => out.push_str(&format!("\x1b[38;2;{r};{g};{b}m▄")),
        (None, None) => out.push(' '),
    }
}

fn visible_rgb(pixel: Rgba<u8>) -> Option<[u8; 3]> {
    let [r, g, b, a] = pixel.0;
    (a >= 16).then_some([r, g, b])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_kitty() {
        let backend = detect_from_pairs([("TERM", "xterm-kitty"), ("COLORTERM", "truecolor")]);
        assert_eq!(backend, RenderBackend::Kitty);
    }

    #[test]
    fn detects_windows_terminal_as_sixel() {
        let backend = detect_from_pairs([("WT_SESSION", "abc"), ("TERM", "xterm-256color")]);
        assert_eq!(backend, RenderBackend::Sixel);
    }

    #[test]
    fn falls_back_to_halfblocks_for_truecolor() {
        let backend = detect_from_pairs([("TERM", "xterm-256color"), ("COLORTERM", "truecolor")]);
        assert_eq!(backend, RenderBackend::Halfblocks);
    }
}
