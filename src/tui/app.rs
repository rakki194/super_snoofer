#![warn(clippy::all, clippy::pedantic)]

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::io::{self, stdout};

use crate::ollama::OllamaClient;

#[derive(Clone)]
pub struct UiState {
    pub input: String,
    pub cursor_position: usize,
    pub scroll: u16,
    pub thinking_visible: bool,
    pub thinking_text: String,
    pub response_text: String,
    pub last_response: Option<String>,
    pub use_codestral: bool,
    pub code_model: String,
    pub standard_model: String,
}

pub struct TuiApp {
    pub state: UiState,
    pub ollama: OllamaClient,
    terminal: ratatui::Terminal<CrosstermBackend<io::Stdout>>,
}

impl TuiApp {
    pub fn new(ollama: OllamaClient, use_codestral: bool) -> Result<Self> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = ratatui::Terminal::new(backend)?;

        // Get model names from the ollama client config
        let standard_model = ollama.model_config.standard_model.clone();
        let code_model = ollama.model_config.code_model.clone();

        let state = UiState {
            input: String::new(),
            cursor_position: 0,
            scroll: 0,
            thinking_visible: true,
            thinking_text: String::new(),
            response_text: String::new(),
            last_response: None,
            use_codestral,
            code_model,
            standard_model,
        };

        Ok(Self {
            state,
            ollama,
            terminal,
        })
    }

    pub fn draw<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Frame)
    {
        self.terminal.draw(f)?;
        Ok(())
    }

    pub fn move_cursor_left(&mut self) {
        self.state.cursor_position = self.state.cursor_position.saturating_sub(1);
    }

    pub fn move_cursor_right(&mut self) {
        if self.state.cursor_position < self.state.input.len() {
            self.state.cursor_position += 1;
        }
    }

    pub fn enter_char(&mut self, c: char) {
        self.state.input.insert(self.state.cursor_position, c);
        self.state.cursor_position += 1;
    }

    pub fn delete_char(&mut self) {
        if self.state.cursor_position > 0 {
            self.state.cursor_position -= 1;
            self.state.input.remove(self.state.cursor_position);
        }
    }

    pub fn delete_char_forward(&mut self) {
        if self.state.cursor_position < self.state.input.len() {
            self.state.input.remove(self.state.cursor_position);
        }
    }

    pub async fn submit_prompt(&mut self) -> Result<()> {
        if self.state.input.is_empty() {
            return Ok(());
        }

        self.state.thinking_text = "🤔 Starting...".to_string();
        self.state.thinking_visible = true;
        self.state.response_text.clear();
        
        let prompt = self.state.input.clone();
        
        // Get response from Ollama with model selection
        match self.ollama.generate_response(&prompt, self.state.use_codestral).await {
            Ok(response) => {
                self.state.thinking_text = "💭 Streaming response...".to_string();
                self.state.response_text = response;
                self.state.thinking_text = "✨ Done!".to_string();
            }
            Err(e) => {
                self.state.thinking_text = "❌ Error".to_string();
                self.state.response_text = format!("Error: {}", e);
            }
        }

        self.state.last_response = Some(self.state.response_text.clone());
        self.state.thinking_visible = true;  // Keep status visible to show completion state
        Ok(())
    }
}

impl Drop for TuiApp {
    fn drop(&mut self) {
        // Restore terminal state
        disable_raw_mode().unwrap_or(());
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        ).unwrap_or(());
        self.terminal.show_cursor().unwrap_or(());
    }
}

pub fn draw_ui(f: &mut Frame, app: &UiState) {
    // Create a flexbox-like layout with the response taking most space
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),     // Input box - fixed height
            Constraint::Length(3),     // Status bar - fixed height
            Constraint::Min(15),       // Response area - takes remaining space with minimum height
        ])
        .margin(1)                     // Add margin around all elements
        .split(f.area());

    // Input box with instructions
    let input_text = format!("{}\nPress Esc to exit, Enter to submit", app.input);
    let input = Paragraph::new(input_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Input (type your question)"))
        .wrap(ratatui::widgets::Wrap { trim: true });  // Enable word wrap for input
    f.render_widget(input, chunks[0]);

    // Set cursor position
    f.set_cursor_position((
        chunks[0].x + app.cursor_position as u16 + 1,
        chunks[0].y + 1,
    ));

    // Status bar with dynamic status
    let (status_text, status_icon) = if app.thinking_visible {
        if app.thinking_text.contains("Starting") {
            (app.thinking_text.as_str(), "🤔")
        } else if app.thinking_text.contains("Streaming") {
            (app.thinking_text.as_str(), "💭")
        } else if app.thinking_text.contains("Done") {
            (app.thinking_text.as_str(), "✨")
        } else if app.thinking_text.contains("Error") {
            (app.thinking_text.as_str(), "❌")
        } else {
            ("Ready for input", "✨")
        }
    } else {
        ("Ready for input", "✨")
    };

    let status = Paragraph::new(status_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!("Status {}", status_icon)));
    f.render_widget(status, chunks[1]);

    // Response area with model indicator and word wrap
    // Get the model name from the OllamaClient's model_config
    let model_name = if app.use_codestral {
        &app.code_model
    } else {
        &app.standard_model
    };
    
    let model_icon = if app.use_codestral { "🧠" } else { "🐬" };
    let response = Paragraph::new(app.response_text.as_str())
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!("Response ({} {})", model_icon, model_name)))
        .wrap(ratatui::widgets::Wrap { trim: true })    // Enable word wrap for response
        .scroll((0, app.scroll));                       // Enable scrolling
    f.render_widget(response, chunks[2]);
} 