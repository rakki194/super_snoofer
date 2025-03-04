use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, ScrollbarState},
    Frame,
};
use std::io::{self, stdout};

use crate::ollama::OllamaClient;

pub struct TuiApp {
    pub input: String,
    pub cursor_position: usize,
    pub scroll: u16,
    pub thinking_visible: bool,
    pub thinking_text: String,
    pub response_text: String,
    pub ollama: OllamaClient,
    pub last_response: Option<String>,
    pub use_codestral: bool,
}

impl TuiApp {
    pub fn new(ollama: OllamaClient, use_codestral: bool) -> Self {
        Self {
            input: String::new(),
            cursor_position: 0,
            scroll: 0,
            thinking_visible: true,
            thinking_text: String::new(),
            response_text: String::new(),
            ollama,
            last_response: None,
            use_codestral,
        }
    }

    pub fn move_cursor_left(&mut self) {
        self.cursor_position = self.cursor_position.saturating_sub(1);
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }

    pub fn enter_char(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input.remove(self.cursor_position);
        }
    }

    pub fn delete_char_forward(&mut self) {
        if self.cursor_position < self.input.len() {
            self.input.remove(self.cursor_position);
        }
    }

    pub async fn submit_prompt(&mut self) -> Result<()> {
        if self.input.is_empty() {
            return Ok(());
        }

        self.thinking_text = "🤔 Thinking...".to_string();
        self.thinking_visible = true;

        let prompt = self.input.clone();
        self.response_text.clear();
        
        // Get response from Ollama with model selection
        match self.ollama.generate_response(&prompt, self.use_codestral).await {
            Ok(response) => {
                self.response_text = response;
            }
            Err(e) => {
                self.response_text = format!("Error: {}", e);
            }
        }

        self.last_response = Some(self.response_text.clone());
        self.thinking_visible = false;
        Ok(())
    }

    pub fn toggle_thinking(&mut self) {
        self.thinking_visible = !self.thinking_visible;
    }
}

pub fn draw_ui(f: &mut Frame, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Input box
    let input = Paragraph::new(app.input.as_str())
        .block(Block::default().borders(Borders::ALL).title("Input"))
        .scroll((0, app.scroll));
    f.render_widget(input, chunks[0]);

    // Set cursor position
    f.set_cursor_position((
        chunks[0].x + app.cursor_position as u16 + 1,
        chunks[0].y + 1,
    ));

    // Thinking area (collapsible)
    if app.thinking_visible {
        let thinking = Paragraph::new(app.thinking_text.as_str())
            .block(Block::default().borders(Borders::ALL).title("Thinking 🤔"));
        f.render_widget(thinking, chunks[1]);
    }

    // Response area with scrollbar
    let response = Paragraph::new(app.response_text.as_str())
        .block(Block::default().borders(Borders::ALL).title("Response 🐬"));
    f.render_widget(response, chunks[2]);

    let mut scrollbar_state = ScrollbarState::default();
    f.render_stateful_widget(
        ratatui::widgets::Scrollbar::default()
            .orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight),
        chunks[2],
        &mut scrollbar_state,
    );
}

pub fn ui(f: &mut Frame, text: &str, scroll_state: &mut ScrollbarState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(f.area());

    let title_block = Block::default()
        .title("Super Snoofer")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let text_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    f.set_cursor_position((chunks[0].x + 1, chunks[0].y + 1));

    let paragraph = Paragraph::new(text)
        .block(text_block)
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(title_block, chunks[0]);
    f.render_widget(paragraph, chunks[1]);

    let scrollbar = ratatui::widgets::Scrollbar::default()
        .orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None);

    f.render_stateful_widget(scrollbar, chunks[2], scroll_state);
}

pub struct TerminalUI {
    terminal: ratatui::Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalUI {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = ratatui::Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn draw<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Frame),
    {
        self.terminal.draw(f)?;
        Ok(())
    }
}

impl Drop for TerminalUI {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        ).unwrap();
        self.terminal.show_cursor().unwrap();
    }
} 