use std::{
    io::{self, Write},
    thread,
    time::Duration,
};

use anyhow::Result;
use clap::ValueEnum;
use crossterm::{
    cursor,
    execute,
    terminal::{Clear, ClearType},
};

const GLITCH_CHARS: &[char] = &['#', '@', '%', '&', '?', '+', '=', '*', '/', '<', '>'];

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TextEffect {
    Typewriter,
    Wipe,
    Glitch,
    Burst,
}

pub fn play_text(
    text: &str,
    effect: TextEffect,
    fps: u64,
    frames: usize,
    no_color: bool,
) -> Result<()> {
    let total_frames = frame_count(frames, text);
    let frame_delay = Duration::from_millis(1000 / fps.max(1));
    let mut stdout = io::stdout();

    execute!(stdout, cursor::Hide)?;

    let result = (|| -> Result<()> {
        for frame in 0..total_frames {
            let rendered = render_ansi_frame(effect, text, frame, total_frames, !no_color);
            execute!(stdout, cursor::MoveToColumn(0), Clear(ClearType::CurrentLine))?;
            write!(stdout, "{rendered}")?;
            stdout.flush()?;
            thread::sleep(frame_delay);
        }

        execute!(stdout, cursor::MoveToColumn(0), Clear(ClearType::CurrentLine))?;
        writeln!(stdout, "{text}")?;
        stdout.flush()?;
        Ok(())
    })();

    execute!(stdout, cursor::Show)?;
    result
}

pub fn render_plain_frame(
    effect: TextEffect,
    text: &str,
    frame: usize,
    total_frames: usize,
) -> String {
    if text.is_empty() {
        return String::new();
    }

    if frame + 1 >= total_frames {
        return text.to_string();
    }

    match effect {
        TextEffect::Typewriter => render_typewriter(text, frame, total_frames),
        TextEffect::Wipe => render_wipe(text, frame, total_frames),
        TextEffect::Glitch => render_glitch(text, frame, total_frames),
        TextEffect::Burst => render_burst(text, frame, total_frames),
    }
}

pub fn render_ansi_frame(
    effect: TextEffect,
    text: &str,
    frame: usize,
    total_frames: usize,
    color: bool,
) -> String {
    let plain = render_plain_frame(effect, text, frame, total_frames);
    if !color || plain.is_empty() {
        return plain;
    }

    let (r, g, b) = effect_color(effect, frame, total_frames);
    format!("\x1b[38;2;{r};{g};{b}m{plain}\x1b[0m")
}

pub fn frame_count(requested: usize, text: &str) -> usize {
    requested.max(text.chars().count()).max(1)
}

fn render_typewriter(text: &str, frame: usize, total_frames: usize) -> String {
    let total_chars = text.chars().count();
    let visible = scaled_count(total_chars, frame, total_frames);
    let mut out = text.chars().take(visible).collect::<String>();

    if visible < total_chars {
        out.push('_');
    }

    out
}

fn render_wipe(text: &str, frame: usize, total_frames: usize) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let visible = scaled_count(chars.len(), frame, total_frames);
    let mut out = String::with_capacity(chars.len());

    for (index, ch) in chars.iter().enumerate() {
        if index < visible {
            out.push(*ch);
        } else if index == visible {
            out.push('>');
        } else if ch.is_whitespace() {
            out.push(' ');
        } else {
            out.push('.');
        }
    }

    out
}

fn render_glitch(text: &str, frame: usize, total_frames: usize) -> String {
    let total_chars = text.chars().count();
    let visible = scaled_count(total_chars, frame, total_frames);
    let mut out = String::with_capacity(text.len());

    for (index, ch) in text.chars().enumerate() {
        if ch.is_whitespace() || index < visible {
            out.push(ch);
        } else {
            out.push(glitch_char(index, frame));
        }
    }

    out
}

fn render_burst(text: &str, frame: usize, total_frames: usize) -> String {
    const SHELLS: &[(&str, &str)] = &[
        ("", ""),
        (". ", " ."),
        ("* ", " *"),
        ("+* ", " *+"),
        (">+* ", " *+<"),
    ];

    let index = if total_frames <= 1 {
        SHELLS.len() - 1
    } else {
        (frame * SHELLS.len() / total_frames).min(SHELLS.len() - 1)
    };
    let (left, right) = SHELLS[index];

    format!("{left}{text}{right}")
}

fn scaled_count(total: usize, frame: usize, total_frames: usize) -> usize {
    if total == 0 {
        return 0;
    }

    let frames = total_frames.max(1);
    (((frame + 1) * total) / frames).min(total)
}

fn glitch_char(index: usize, frame: usize) -> char {
    let value = index
        .wrapping_mul(31)
        .wrapping_add(frame.wrapping_mul(17))
        .wrapping_add(index ^ frame);
    GLITCH_CHARS[value % GLITCH_CHARS.len()]
}

fn effect_color(effect: TextEffect, frame: usize, total_frames: usize) -> (u8, u8, u8) {
    let phase = if total_frames <= 1 {
        0.0
    } else {
        frame as f32 / (total_frames - 1) as f32
    };

    match effect {
        TextEffect::Typewriter => (255, (190.0 + 45.0 * phase) as u8, 80),
        TextEffect::Wipe => (80, (210.0 + 35.0 * phase) as u8, 255),
        TextEffect::Glitch => (255, 80, (160.0 + 80.0 * phase) as u8),
        TextEffect::Burst => (255, (130.0 + 90.0 * phase) as u8, 80),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typewriter_reveals_text_over_time() {
        assert_eq!(
            render_plain_frame(TextEffect::Typewriter, "hello", 0, 5),
            "h_"
        );
        assert_eq!(
            render_plain_frame(TextEffect::Typewriter, "hello", 4, 5),
            "hello"
        );
    }

    #[test]
    fn wipe_uses_ascii_placeholder_cells() {
        assert_eq!(
            render_plain_frame(TextEffect::Wipe, "hi", 0, 4),
            ">."
        );
    }

    #[test]
    fn glitch_finishes_on_original_text() {
        assert_eq!(
            render_plain_frame(TextEffect::Glitch, "ship it", 9, 10),
            "ship it"
        );
    }

    #[test]
    fn burst_wraps_text_with_ascii_shells() {
        assert_eq!(
            render_plain_frame(TextEffect::Burst, "ok", 2, 10),
            ". ok ."
        );
    }
}
