#![warn(clippy::all, clippy::pedantic)]

use crate::ollama::{ModelConfig, OllamaClient};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

mod app;

pub use app::{TuiApp, draw_ui};

/// Run the terminal user interface mode
/// 
/// # Errors
/// Returns an error if the TUI cannot be initialized or if there's an error during execution
pub async fn run_tui_mode(prompt: &str, use_codestral: bool, model_config: ModelConfig) -> Result<()> {
    // Initialize terminal with custom model configuration
    let ollama = OllamaClient::with_config(model_config);
    let mut app = TuiApp::new(ollama, use_codestral)?;
    
    // Prefill the prompt if provided
    if !prompt.is_empty() {
        app.state.input = prompt.to_string();
        app.state.cursor_position = prompt.len();
    }

    // Event loop
    loop {
        // Draw the UI
        let state = app.state.clone();
        app.draw(|f| draw_ui(f, &state))?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Enter => {
                        app.submit_prompt().await?;
                    }
                    KeyCode::Char(c) => {
                        app.enter_char(c);
                    }
                    KeyCode::Backspace => {
                        app.delete_char();
                    }
                    KeyCode::Delete => {
                        app.delete_char_forward();
                    }
                    KeyCode::Left => {
                        app.move_cursor_left();
                    }
                    KeyCode::Right => {
                        app.move_cursor_right();
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
