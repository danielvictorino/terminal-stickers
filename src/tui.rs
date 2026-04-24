use std::{path::Path, time::Duration};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Terminal,
};

use crate::{
    animation::{self, TextEffect},
    cli::ChatArgs,
    manifest::Sticker,
    packs::{all_stickers, discover_packs},
};

pub fn run_chat(pack_dir: &Path, _args: ChatArgs) -> Result<()> {
    let packs = discover_packs(pack_dir)?;
    let stickers = all_stickers(&packs);
    let mut app = ChatApp::new(stickers);

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut ChatApp,
) -> Result<()> {
    loop {
        terminal.draw(|frame| draw(frame, app))?;

        if event::poll(Duration::from_millis(120))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && app.handle_key(key.code) {
                    break;
                }
            }
        }

        app.tick();
    }

    Ok(())
}

fn draw(frame: &mut ratatui::Frame<'_>, app: &ChatApp) {
    let area = frame.area();
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(area);
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(outer[0]);

    let stickers = app.visible_stickers();
    let items = stickers
        .iter()
        .enumerate()
        .map(|(index, sticker)| {
            let style = if index == app.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::raw(sticker.qualified_id()),
                Span::raw("  "),
                Span::styled(sticker.name.clone(), Style::default().fg(Color::White)),
            ]))
            .style(style)
        })
        .collect::<Vec<_>>();

    let sticker_list = List::new(items).block(
        Block::default()
            .title("Terminal Stickers")
            .borders(Borders::ALL),
    );
    frame.render_widget(sticker_list, panes[0]);

    let mut messages = if app.messages.is_empty() {
        Vec::from([Line::from("No stickers sent yet.")])
    } else {
        app.messages
            .iter()
            .map(|message| Line::from(message.as_str()))
            .collect()
    };

    if let Some(animation) = app.animation_line() {
        messages.insert(
            0,
            Line::styled(
                animation,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        );
    }
    let chat = Paragraph::new(messages)
        .block(Block::default().title("Conversation").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    frame.render_widget(chat, panes[1]);

    let search = Paragraph::new(app.search_line())
        .block(Block::default().title("Search").borders(Borders::ALL));
    frame.render_widget(search, outer[1]);
}

#[derive(Debug)]
struct ChatApp {
    stickers: Vec<Sticker>,
    selected: usize,
    search: String,
    searching: bool,
    messages: Vec<String>,
    animation: Option<ActiveAnimation>,
}

impl ChatApp {
    fn new(stickers: Vec<Sticker>) -> Self {
        Self {
            stickers,
            selected: 0,
            search: String::new(),
            searching: false,
            messages: Vec::new(),
            animation: None,
        }
    }

    fn visible_stickers(&self) -> Vec<&Sticker> {
        self.stickers
            .iter()
            .filter(|sticker| self.search.is_empty() || sticker.matches_query(&self.search))
            .collect()
    }

    fn search_line(&self) -> String {
        if self.searching {
            format!("{}_", self.search)
        } else {
            self.search.clone()
        }
    }

    fn handle_key(&mut self, code: KeyCode) -> bool {
        if self.searching {
            match code {
                KeyCode::Esc | KeyCode::Enter => self.searching = false,
                KeyCode::Backspace => {
                    self.search.pop();
                    self.clamp_selection();
                }
                KeyCode::Char(ch) => {
                    self.search.push(ch);
                    self.clamp_selection();
                }
                _ => {}
            }
            return false;
        }

        match code {
            KeyCode::Char('q') | KeyCode::Esc => true,
            KeyCode::Char('/') => {
                self.searching = true;
                false
            }
            KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
                false
            }
            KeyCode::Down => {
                let max = self.visible_stickers().len().saturating_sub(1);
                self.selected = (self.selected + 1).min(max);
                false
            }
            KeyCode::Enter => {
                self.send_selected();
                false
            }
            _ => false,
        }
    }

    fn clamp_selection(&mut self) {
        let max = self.visible_stickers().len().saturating_sub(1);
        self.selected = self.selected.min(max);
    }

    fn send_selected(&mut self) {
        let Some(sticker) = self.visible_stickers().get(self.selected).copied().cloned() else {
            return;
        };

        self.messages
            .push(format!("me: [{}] {}", sticker.qualified_id(), sticker.name));
        self.animation = Some(ActiveAnimation::new(
            format!("sent {}", sticker.name),
            TextEffect::Burst,
            18,
        ));
    }

    fn animation_line(&self) -> Option<String> {
        self.animation.as_ref().map(ActiveAnimation::line)
    }

    fn tick(&mut self) {
        if let Some(animation) = self.animation.as_mut() {
            animation.frame += 1;
            if animation.frame >= animation.frames {
                self.animation = None;
            }
        }
    }
}

#[derive(Debug)]
struct ActiveAnimation {
    text: String,
    effect: TextEffect,
    frame: usize,
    frames: usize,
}

impl ActiveAnimation {
    fn new(text: String, effect: TextEffect, frames: usize) -> Self {
        Self {
            text,
            effect,
            frame: 0,
            frames,
        }
    }

    fn line(&self) -> String {
        animation::render_plain_frame(self.effect, &self.text, self.frame, self.frames)
    }
}
