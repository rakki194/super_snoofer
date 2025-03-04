#![warn(clippy::all, clippy::pedantic)]

use crate::ollama::{ModelConfig, OllamaClient};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseEventKind, MouseButton, EnableMouseCapture, DisableMouseCapture, KeyModifiers};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use std::time::Duration;
use tokio::time::sleep;
use std::io;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;

mod app;

pub use app::{TuiApp, draw_ui, ModelState};

/// Messages sent between the UI and background tasks
pub enum UiMessage {
    /// Update the response text in the UI
    ResponseUpdate(String),
    /// An error occurred while processing the request
    Error(String),
    /// Streaming response completed
    StreamingComplete,
}

/// Get a client for Ollama API
pub fn get_ollama_client() -> OllamaClient {
    OllamaClient::with_config(ModelConfig::default())
}

/// Get a client for OpenAI API (could be a placeholder until implemented)
pub fn get_openai_client() -> OllamaClient {
    // TODO: Replace with actual OpenAI client
    OllamaClient::with_config(ModelConfig::default())
}

/// Run the terminal user interface mode
/// 
/// # Errors
/// Returns an error if the TUI cannot be initialized or if there's an error during execution
pub async fn run_tui_mode(prompt: &str, use_codestral: bool, model_config: ModelConfig) -> Result<()> {
    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture // Enable mouse for selection
    )?;
    
    // Create terminal and app
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    
    // Create app with the specified model and config
    let ollama = OllamaClient::with_config(model_config);
    let standard_model = if use_codestral { "codestral" } else { "standard-model" };
    let code_model = "codestral";
    
    let (tx, rx) = mpsc::channel(100);
    
    let mut app = TuiApp::with_terminal(
        ollama,
        terminal,
        standard_model.to_string(),
        code_model.to_string(),
        tx,
    )?;
    
    // Prefill the prompt if provided
    if !prompt.is_empty() {
        app.state.input = prompt.to_string();
        app.state.cursor_position = prompt.len();
        app.update_input_height();
    }
    
    // Run the main application
    let result = run_app(app, rx).await;
    
    // Clean up terminal before returning
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    
    result
}

pub async fn run_ui(mut app: TuiApp) -> io::Result<()> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture // Enable mouse for selection
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main event loop
    loop {
        // Draw UI
        terminal.draw(|_f| {
            let state = app.state.clone();
            if let Err(e) = app.draw(|frame| draw_ui(frame, &state)) {
                eprintln!("Error drawing UI: {}", e);
            }
        })?;

        // Handle input events with timeout to allow for streaming updates
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => {
                        if app.state.selection_mode {
                            // Exit selection mode if active
                            app.toggle_selection_mode();
                        } else if app.state.loading || app.state.model_state == ModelState::Streaming {
                            // Cancel the current response if one is in progress
                            app.request_cancel();
                        } else {
                            // Exit the application if not loading
                            break;
                        }
                    }
                    KeyCode::Enter => {
                        if app.state.selection_mode {
                            // In selection mode, Enter copies selected text
                            // Clipboard handling would go here if supported
                            app.toggle_selection_mode();
                        } else if key.modifiers.contains(KeyModifiers::SHIFT) {
                            // Shift+Enter adds a newline instead of submitting
                            app.add_newline();
                        } else {
                            // Normal Enter submits the current input
                            if let Err(e) = app.submit_prompt().await {
                                eprintln!("Error submitting prompt: {}", e);
                            }
                        }
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Exit on Ctrl+C
                        break;
                    }
                    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Ctrl+S toggles selection mode
                        app.toggle_selection_mode();
                    }
                    KeyCode::F(1) => {
                        // F1 toggles thinking sections
                        app.toggle_thinking_sections();
                    }
                    KeyCode::Char(c) => {
                        // Add character to input
                        app.add_char(c);
                        app.update_input_height();
                    }
                    KeyCode::Backspace => {
                        // Remove character from input
                        app.delete_char();
                        app.update_input_height();
                    }
                    KeyCode::Delete => {
                        // Forward delete
                        app.forward_delete_char();
                        app.update_input_height();
                    }
                    KeyCode::Left => {
                        // Move cursor left
                        app.move_cursor_left();
                    }
                    KeyCode::Right => {
                        // Move cursor right
                        app.move_cursor_right();
                    }
                    KeyCode::Up => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            // Ctrl+Up decreases scroll by 1
                            app.scroll_up();
                        } else {
                            // Move input cursor up a line if multi-line
                            app.move_cursor_up();
                        }
                    }
                    KeyCode::Down => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            // Ctrl+Down increases scroll by 1
                            app.scroll_down();
                        } else {
                            // Move input cursor down a line if multi-line
                            app.move_cursor_down();
                        }
                    }
                    KeyCode::PageUp => {
                        // Page up - scroll by a large amount
                        let page_size = 10; // Or calculate based on terminal size
                        app.page_up(page_size);
                    }
                    KeyCode::PageDown => {
                        // Page down - scroll by a large amount
                        let page_size = 10; // Or calculate based on terminal size
                        app.page_down(page_size);
                    }
                    KeyCode::Home => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            // Ctrl+Home scrolls to the top
                            app.scroll_to_top();
                        } else {
                            // Regular Home moves cursor to the start of the line
                            app.move_cursor_to_start_of_line();
                        }
                    }
                    KeyCode::End => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            // Ctrl+End scrolls to the bottom
                            app.scroll_to_bottom();
                        } else {
                            // Regular End moves cursor to the end of the line
                            app.move_cursor_to_end_of_line();
                        }
                    }
                    _ => {}
                }
            } else if let Event::Mouse(mouse) = event::read()? {
                match mouse.kind {
                    MouseEventKind::ScrollDown => {
                        // Mouse wheel down - scroll down
                        app.scroll_down();
                    }
                    MouseEventKind::ScrollUp => {
                        // Mouse wheel up - scroll up
                        app.scroll_up();
                    }
                    MouseEventKind::Down(MouseButton::Left) => {
                        // Left click - handle based on area
                        if let Ok((terminal_cols, terminal_rows)) = app.get_terminal_size() {
                            // Calculate the layout similar to draw_ui
                            let input_height = app.state.input_height;
                            let status_height = 3;
                            let margin = 1;
                            
                            // Simple layout calculation
                            let input_area_top = margin; 
                            let input_area_bottom = input_area_top + input_height;
                            
                            let status_area_top = input_area_bottom;
                            let status_area_bottom = status_area_top + status_height;
                            
                            let response_area_top = status_area_bottom;
                            let response_area_bottom = terminal_rows - margin;
                            
                            let scrollbar_column = terminal_cols - margin - 1; // Last visible column
                            
                            // Check if click is on scrollbar
                            if mouse.column >= scrollbar_column && 
                               mouse.row > response_area_top && 
                               mouse.row < response_area_bottom && 
                               app.state.scroll_max > 0 {
                                
                                // Calculate relative position in scrollbar to determine scroll position
                                let scrollbar_height = response_area_bottom - response_area_top - 2; // -2 for borders
                                let relative_pos = mouse.row - (response_area_top + 1);
                                let scroll_ratio = f64::from(relative_pos) / f64::from(scrollbar_height);
                                let scroll_position = 
                                    (f64::from(app.state.scroll_max) * scroll_ratio).round() as u16;
                                
                                // Set new scroll position
                                app.state.scroll = scroll_position;
                            } 
                            // Check if click is in response area (for selection)
                            else if mouse.row > response_area_top && 
                                    mouse.row < response_area_bottom &&
                                    app.state.selection_mode {
                                // Handle text selection - not fully implemented
                                // Would need to track selection start and end positions
                            }
                            // Check if click is in input area (for cursor positioning)
                            else if mouse.row > input_area_top && 
                                    mouse.row < input_area_bottom {
                                // Would position cursor based on click - not fully implemented
                                // Requires mapping screen coordinates to text position
                            }
                        }
                    },
                    _ => {}
                }
            }
        }

        // Check if app state has changed since last UI update
        if app.has_updates() {
            // Recalculate scrollbar max value based on content length and visible height
            if let Ok((_, terminal_rows)) = app.get_terminal_size() {
                // Calculate the response view height
                let response_view_height = terminal_rows.saturating_sub(
                    app.state.input_height + 3 + 2 // 3 for status bar, 2 for margins
                );
                
                // Count the number of lines in the response text
                let response_line_count = app.state.response_text.lines().count();
                
                // Set the maximum scroll value
                if response_line_count > response_view_height as usize {
                    app.state.scroll_max = (response_line_count - response_view_height as usize) as u16;
                } else {
                    app.state.scroll_max = 0;
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

/// Run the terminal UI application with message handling
pub async fn run_app(mut app: TuiApp, rx: mpsc::Receiver<UiMessage>) -> Result<()> {
    let mut message_rx = rx;
    
    // Main event loop
    loop {
        // Draw UI
        let state = app.state.clone();
        app.draw(|frame| draw_ui(frame, &state))?;

        // Poll for messages or events with timeout
        let mut event_received = false;
        
        // First, check for UI messages (streaming responses, etc.)
        if let Ok(message) = tokio::time::timeout(Duration::from_millis(10), message_rx.recv()).await {
            if let Some(message) = message {
                match message {
                    UiMessage::ResponseUpdate(text) => {
                        app.state.response_text = text;
                        let view_height = app.get_response_view_height();
                        app.update_scroll_max(view_height);
                    },
                    UiMessage::Error(error) => {
                        app.state.response_text = format!("Error: {}", error);
                        app.state.is_streaming = false;
                    },
                    UiMessage::StreamingComplete => {
                        app.state.is_streaming = false;
                        // If there was saved input, restore it
                        if !app.state.saved_input.is_empty() {
                            app.state.input = app.state.saved_input.clone();
                            app.state.cursor_position = app.state.input.len();
                            app.state.saved_input.clear();
                        }
                    },
                }
                event_received = true;
            }
        }
        
        // Then, poll for user input events
        if event::poll(Duration::from_millis(if event_received { 0 } else { 50 }))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    match key.code {
                        KeyCode::Esc => {
                            if app.state.selection_mode {
                                // Exit selection mode if active
                                app.toggle_selection_mode();
                            } else if app.state.is_streaming {
                                // Cancel the current response if streaming
                                let cancel_requested = app.get_cancel_requested();
                                if let Ok(mut guard) = cancel_requested.lock() {
                                    *guard = true;
                                }
                                
                                let cancel_flag = app.get_cancel_flag();
                                if let Ok(mut guard) = cancel_flag.lock() {
                                    *guard = true;
                                }
                                
                                app.state.cancel_requested = true;
                            } else {
                                // Exit the application
                                break;
                            }
                        },
                        KeyCode::Enter => {
                            if app.state.selection_mode {
                                // In selection mode, Enter conceptually copies selected text
                                app.toggle_selection_mode();
                            } else if key.modifiers.contains(KeyModifiers::SHIFT) {
                                // Shift+Enter adds a newline instead of submitting
                                app.add_newline();
                            } else if !app.state.is_streaming {
                                // Normal Enter submits the current input
                                if let Err(e) = app.submit_prompt().await {
                                    eprintln!("Error submitting prompt: {}", e);
                                }
                            }
                        },
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            // Exit on Ctrl+C
                            break;
                        },
                        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            // Ctrl+S toggles selection mode
                            app.toggle_selection_mode();
                        },
                        KeyCode::F(1) => {
                            // F1 toggles thinking sections
                            app.toggle_thinking_sections();
                        },
                        KeyCode::Char(c) => {
                            // Add character to input
                            app.add_char(c);
                        },
                        KeyCode::Backspace => {
                            // Remove character from input
                            app.delete_char();
                        },
                        KeyCode::Delete => {
                            // Forward delete
                            app.forward_delete_char();
                        },
                        KeyCode::Left => {
                            // Move cursor left
                            app.move_cursor_left();
                        },
                        KeyCode::Right => {
                            // Move cursor right
                            app.move_cursor_right();
                        },
                        KeyCode::Up => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                // Ctrl+Up decreases scroll by 1
                                app.scroll_up();
                            } else {
                                // Move input cursor up a line if multi-line
                                app.move_cursor_up();
                            }
                        },
                        KeyCode::Down => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                // Ctrl+Down increases scroll by 1
                                app.scroll_down();
                            } else {
                                // Move input cursor down a line if multi-line
                                app.move_cursor_down();
                            }
                        },
                        KeyCode::PageUp => {
                            // Page up - scroll by a large amount
                            let page_size = 10; // Or calculate based on terminal size
                            app.page_up(page_size);
                        },
                        KeyCode::PageDown => {
                            // Page down - scroll by a large amount
                            let page_size = 10; // Or calculate based on terminal size
                            app.page_down(page_size);
                        },
                        KeyCode::Home => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                // Ctrl+Home scrolls to the top
                                app.scroll_to_top();
                            } else {
                                // Regular Home moves cursor to the start of the line
                                app.move_cursor_to_start_of_line();
                            }
                        },
                        KeyCode::End => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                // Ctrl+End scrolls to the bottom
                                app.scroll_to_bottom();
                            } else {
                                // Regular End moves cursor to the end of the line
                                app.move_cursor_to_end_of_line();
                            }
                        },
                        _ => {}
                    }
                },
                Event::Mouse(mouse) => {
                    match mouse.kind {
                        MouseEventKind::ScrollDown => {
                            // Mouse wheel down - scroll down
                            app.scroll_down();
                        },
                        MouseEventKind::ScrollUp => {
                            // Mouse wheel up - scroll up
                            app.scroll_up();
                        },
                        MouseEventKind::Down(MouseButton::Left) => {
                            // Left mouse button down - handle clicking on the scrollbar
                            let (width, _height) = get_terminal_size()?;
                            
                            // Check if click is on the scrollbar area (rightmost 2 columns)
                            if mouse.column >= width.saturating_sub(2) {
                                // Calculate the click position relative to the scrollbar
                                let response_height = app.get_response_view_height();
                                let relative_click = mouse.row as f64 / response_height as f64;
                                
                                // Set scroll position based on click
                                app.set_scroll_percentage(relative_click as f32);
                            } else {
                                // Click anywhere else enters selection mode
                                if !app.state.selection_mode {
                                    app.toggle_selection_mode();
                                }
                                // TODO: Update selection start position
                            }
                        },
                        _ => {}
                    }
                },
                Event::Resize(_width, _height) => {
                    // Terminal was resized - update layout
                    app.handle_resize()?;
                },
                _ => {}
            }
        }
        
        // Short sleep to prevent CPU hogging
        sleep(Duration::from_millis(5)).await;
    }
    
    Ok(())
}

/// Get the current terminal size
pub fn get_terminal_size() -> anyhow::Result<(u16, u16)> {
    let size = crossterm::terminal::size()?;
    Ok(size)
}
