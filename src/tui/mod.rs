#![warn(clippy::all, clippy::pedantic)]

use crate::ollama::{ModelConfig, OllamaClient};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

mod app;

pub use app::{TuiApp, draw_ui};

/// Run the TUI mode
pub async fn run_tui_mode(prompt: &str, use_codestral: bool, model_config: ModelConfig) -> Result<()> {
    // Initialize terminal with custom model configuration
    let ollama = OllamaClient::with_config(model_config).await?;
    let mut terminal = TuiApp::new(ollama, use_codestral)?;

    // Set the initial prompt
    terminal.state.input = prompt.to_string();
    terminal.state.cursor_position = terminal.state.input.len();

    // Event loop
    loop {
        // Draw the UI
        let state = terminal.state.clone();
        terminal.draw(|f| draw_ui(f, &state))?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Enter => {
                        terminal.submit_prompt().await?;
                    }
                    KeyCode::Char(c) => {
                        terminal.enter_char(c);
                    }
                    KeyCode::Backspace => {
                        terminal.delete_char();
                    }
                    KeyCode::Delete => {
                        terminal.delete_char_forward();
                    }
                    KeyCode::Left => {
                        terminal.move_cursor_left();
                    }
                    KeyCode::Right => {
                        terminal.move_cursor_right();
                    }
                    KeyCode::Esc => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
