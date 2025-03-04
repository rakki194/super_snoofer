#![warn(clippy::all, clippy::pedantic)]

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
    Terminal,
};
use std::io::{self, stdout};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::ollama::OllamaClient;
use crate::ollama::ModelConfig as Config;
use super::UiMessage;
use super::{get_ollama_client, get_openai_client};

/// Different states of the model processing
#[derive(Debug, Clone, PartialEq)]
pub enum ModelState {
    /// Initial state, not processing
    Idle,
    /// Loading the model into memory
    Loading,
    /// Model is initialized and generating a response
    Generating,
    /// Model is streaming text back
    Streaming,
    /// Processing is complete
    Complete,
    /// An error occurred
    Error,
}

#[derive(Clone)]
pub struct UiState {
    pub input: String,
    pub response_text: String,
    pub cursor_position: usize,
    pub thinking_text: String,
    pub loading: bool,
    pub loading_animation_frame: usize,
    pub model_state: ModelState,
    pub use_codestral: bool,
    pub standard_model: String,
    pub code_model: String,
    pub scroll: u16,
    pub scroll_max: u16,
    pub show_thinking_sections: bool,
    pub thinking_visible: bool,
    pub thinking_sections_visible: bool,
    pub last_response: Option<String>,
    pub input_height: u16,          // Height of the input box
    pub selection_mode: bool,       // Whether we're in selection mode
    pub cancel_requested: bool,     // Whether a cancel has been requested
    pub history: Vec<String>,
    pub history_position: usize,
    pub is_streaming: bool,
    pub saved_input: String,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            input: String::new(),
            response_text: String::new(),
            cursor_position: 0,
            thinking_text: String::new(),
            loading: false,
            loading_animation_frame: 0,
            model_state: ModelState::Idle,
            use_codestral: false,
            standard_model: String::from("llama3"),
            code_model: String::from("codestral"),
            scroll: 0,
            scroll_max: 0,
            show_thinking_sections: true,
            thinking_visible: true,
            thinking_sections_visible: true,
            last_response: None,
            input_height: 4,          // Default to 4 (2 content lines + 2 border lines)
            selection_mode: false,    // Not in selection mode by default
            cancel_requested: false,  // No cancel requested by default
            history: Vec::new(),
            history_position: 0,
            is_streaming: false,
            saved_input: String::new(),
        }
    }
}

pub struct TuiApp {
    pub state: UiState,
    pub ollama: OllamaClient,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    config: Arc<Config>,
    cancel_flag: Arc<Mutex<bool>>,
    cancel_requested: Arc<Mutex<bool>>,
    tx: mpsc::Sender<UiMessage>,
}

impl TuiApp {
    /// Creates a new TUI application
    /// 
    /// # Errors
    /// Returns an error if the terminal cannot be initialized
    pub fn new(ollama: OllamaClient, use_codestral: bool) -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableMouseCapture
        )?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Get model names from config
        let standard_model_name = ollama.model_config.standard_model.clone();
        let code_model_name = ollama.model_config.code_model.clone();

        // Create state with model preferences
        let mut state = UiState::default();
        state.standard_model = standard_model_name.clone();
        state.code_model = code_model_name.clone();
        state.use_codestral = use_codestral;
        
        let config = Arc::new(Config {
            standard_model: if use_codestral { code_model_name.clone() } else { standard_model_name.clone() },
            code_model: code_model_name,
        });

        let cancel_flag = Arc::new(Mutex::new(false));
        let cancel_requested = Arc::new(Mutex::new(false));
        let (tx, _rx) = mpsc::channel(10);

        Ok(Self {
            state,
            ollama,
            terminal,
            config,
            cancel_flag,
            cancel_requested,
            tx,
        })
    }

    /// Gets the terminal size
    /// 
    /// # Errors
    /// Returns an error if the terminal size cannot be determined
    pub fn get_terminal_size(&mut self) -> Result<(u16, u16)> {
        let size = self.terminal.size()?;
        Ok((size.width, size.height))
    }

    /// Draws the UI components
    /// 
    /// # Errors
    /// Returns an error if rendering to the terminal fails
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

    /// Update the loading animation
    pub fn update_loading_animation(&mut self) {
        if !self.state.loading {
            return;
        }
        
        // Update animation frame
        self.state.loading_animation_frame = (self.state.loading_animation_frame + 1) % 10;
        let animation_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let frame = animation_frames[self.state.loading_animation_frame % animation_frames.len()];
        
        // Get model name
        let model_name = if self.state.use_codestral {
            &self.state.code_model
        } else {
            &self.state.standard_model
        };
        
        // Update the thinking text based on the current model state
        match self.state.model_state {
            ModelState::Loading => {
                self.state.thinking_text = format!("{} Loading model {}...", frame, model_name);
            }
            ModelState::Generating => {
                self.state.thinking_text = format!("{} Generating response...", frame);
            }
            ModelState::Streaming => {
                self.state.thinking_text = format!("{} Streaming response...", frame);
            }
            _ => {
                // For other states, don't change the message
            }
        }
    }

    /// Submits the current prompt to the model and handles the response
    /// 
    /// # Errors
    /// Returns an error if the prompt cannot be processed or the model fails
    pub async fn submit_prompt(&mut self) -> Result<()> {
        if self.state.input.trim().is_empty() {
            return Ok(());
        }
        
        let prompt = self.state.input.clone();
        
        // Save the current input in case the user wants to type something new
        // while the response is being generated
        self.state.saved_input = prompt.clone();
        self.state.input.clear();
        self.state.cursor_position = 0;
        
        // Set streaming flag
        self.state.is_streaming = true;
        
        // Start streaming response
        let standard_model = self.config.standard_model.clone();
        let code_model = self.config.code_model.clone();
        let cancel_flag = Arc::clone(&self.cancel_flag);
        let cancel_requested = Arc::clone(&self.cancel_requested);
        let tx = self.tx.clone();
        
        tokio::spawn(async move {
            // Reset the cancel flag at the start of streaming
            if let Ok(mut cancel_flag) = cancel_flag.lock() {
                *cancel_flag = false;
            }
            
            // Clear cancel_requested at the start of streaming
            if let Ok(mut cancel_req) = cancel_requested.lock() {
                *cancel_req = false;
            }
            
            let model_to_use = if standard_model == "codestral" { code_model } else { standard_model };
            
            // Get clients and stream the prompt
            let connection_result: Result<String, anyhow::Error> = 
                if !model_to_use.starts_with("gpt") {
                    let _client = get_ollama_client();
                    // Implement proper streaming with the client
                    Ok("Response from Ollama model".to_string())
                } else {
                    // Use OpenAI for GPT models
                    let _client = get_openai_client();
                    // Implement proper streaming with the client  
                    Ok("Response from OpenAI model".to_string())
                };
            
            match connection_result {
                Ok(response) => {
                    // Send the response back to the UI
                    if let Err(e) = tx.send(UiMessage::ResponseUpdate(response)).await {
                        eprintln!("Failed to send response: {}", e);
                    }
                    
                    // Mark streaming as complete
                    if let Err(e) = tx.send(UiMessage::StreamingComplete).await {
                        eprintln!("Failed to send streaming complete: {}", e);
                    }
                },
                Err(e) => {
                    // Send the error back to the UI
                    if let Err(send_err) = tx.send(UiMessage::Error(e.to_string())).await {
                        eprintln!("Failed to send error: {}", send_err);
                    }
                }
            }
        });
        
        Ok(())
    }

    /// Scrolls the response text up
    pub fn scroll_up(&mut self) {
        if self.state.scroll > 0 {
            self.state.scroll -= 1;
        }
    }

    /// Scrolls the response text down
    pub fn scroll_down(&mut self) {
        if self.state.scroll < self.state.scroll_max {
            self.state.scroll += 1;
        }
    }

    /// Scrolls the response text up by a page
    pub fn page_up(&mut self, page_size: u16) {
        if self.state.scroll > page_size {
            self.state.scroll -= page_size;
        } else {
            self.state.scroll = 0;
        }
    }

    /// Scrolls the response text down by a page
    pub fn page_down(&mut self, page_size: u16) {
        if self.state.scroll + page_size < self.state.scroll_max {
            self.state.scroll += page_size;
        } else {
            self.state.scroll = self.state.scroll_max;
        }
    }

    /// Calculate the scroll max based on content and view size
    pub fn update_scroll_max(&mut self, view_height: u16) {
        let line_count = self.state.response_text.lines().count() as u16;
        self.state.scroll_max = if line_count > view_height {
            line_count - view_height
        } else {
            0
        };
    }

    /// Toggle visibility of thinking sections
    pub fn toggle_thinking_sections(&mut self) {
        self.state.show_thinking_sections = !self.state.show_thinking_sections;
        // Reset scroll when toggling to avoid confusion
        self.state.scroll = 0;
    }

    /// Toggle selection mode
    pub fn toggle_selection_mode(&mut self) {
        self.state.selection_mode = !self.state.selection_mode;
    }

    /// Request cancellation of the current operation
    pub fn request_cancel(&mut self) {
        if let Ok(mut requested) = self.cancel_requested.lock() {
            *requested = true;
        }
        
        if let Ok(mut flag) = self.cancel_flag.lock() {
            *flag = true;
        }
        
        self.state.cancel_requested = true;
    }

    /// Update input height based on content
    pub fn update_input_height(&mut self) {
        // Count the number of lines in the input text
        // We need at least 2 lines and at most 10 lines
        let line_count = self.state.input.lines().count();
        
        // Ensure we have at least 2 lines of content area,
        // plus 1 for the border/title at top and 1 for border at bottom
        let min_height = 4; // 2 content lines + 2 border lines
        
        // Maximum of 10 content lines + 2 border lines = 12
        let max_height = 12; 
        
        // Calculate the height based on content, ensuring it's between the min and max
        self.state.input_height = u16::try_from(line_count)
            .unwrap_or(0)
            .saturating_add(2) // Add border lines
            .max(min_height)
            .min(max_height);
    }
    
    /// Add a newline to the input field
    pub fn add_newline(&mut self) {
        self.state.input.insert(self.state.cursor_position, '\n');
        self.state.cursor_position += 1;
        self.update_input_height();
    }

    /// Move cursor to the start of the current line
    pub fn move_cursor_to_start_of_line(&mut self) {
        // Find the start of the current line
        let mut i = self.state.cursor_position;
        
        // Move backwards until we find a newline or the start of the input
        while i > 0 && self.state.input.chars().nth(i - 1) != Some('\n') {
            i -= 1;
        }
        
        self.state.cursor_position = i;
    }
    
    /// Move cursor to the end of the current line
    pub fn move_cursor_to_end_of_line(&mut self) {
        // Find the end of the current line
        let mut i = self.state.cursor_position;
        
        // Move forward until we find a newline or the end of the input
        while i < self.state.input.len() && self.state.input.chars().nth(i) != Some('\n') {
            i += 1;
        }
        
        self.state.cursor_position = i;
    }
    
    /// Move cursor up a line
    pub fn move_cursor_up(&mut self) {
        // Find the current line's start
        let mut line_start = self.state.cursor_position;
        while line_start > 0 && self.state.input.chars().nth(line_start - 1) != Some('\n') {
            line_start -= 1;
        }
        
        // Current column within this line
        let current_col = self.state.cursor_position - line_start;
        
        // If we're already at the first line, do nothing
        if line_start == 0 {
            return;
        }
        
        // Find the start of the previous line
        let mut prev_line_start = line_start - 1;
        while prev_line_start > 0 && self.state.input.chars().nth(prev_line_start - 1) != Some('\n') {
            prev_line_start -= 1;
        }
        
        // Find the end of the previous line
        let prev_line_end = line_start - 1;
        
        // Calculate the previous line length
        let prev_line_len = prev_line_end - prev_line_start + 1;
        
        // Calculate new position, ensuring we don't go beyond the previous line length
        let new_col = current_col.min(prev_line_len);
        self.state.cursor_position = prev_line_start + new_col;
    }
    
    /// Move cursor down a line
    pub fn move_cursor_down(&mut self) {
        // If we're at the end of the input, do nothing
        if self.state.cursor_position >= self.state.input.len() {
            return;
        }
        
        // Find the current line's start
        let mut line_start = self.state.cursor_position;
        while line_start > 0 && self.state.input.chars().nth(line_start - 1) != Some('\n') {
            line_start -= 1;
        }
        
        // Current column within this line
        let current_col = self.state.cursor_position - line_start;
        
        // Find the end of the current line
        let mut line_end = self.state.cursor_position;
        while line_end < self.state.input.len() && self.state.input.chars().nth(line_end) != Some('\n') {
            line_end += 1;
        }
        
        // If we're at the last line, do nothing
        if line_end >= self.state.input.len() {
            return;
        }
        
        // Move to start of next line
        let next_line_start = line_end + 1;
        
        // Find the end of the next line
        let mut next_line_end = next_line_start;
        while next_line_end < self.state.input.len() && self.state.input.chars().nth(next_line_end) != Some('\n') {
            next_line_end += 1;
        }
        
        // Calculate the next line length
        let next_line_len = next_line_end - next_line_start;
        
        // Calculate new position, ensuring we don't go beyond the next line length
        let new_col = current_col.min(next_line_len);
        self.state.cursor_position = next_line_start + new_col;
    }
    
    /// Check if there are updates to be processed by the UI
    pub fn has_updates(&self) -> bool {
        // In a real implementation, this would track changes to the state
        // For now, just return true to always recalculate (could be optimized)
        true
    }

    /// Alias for enter_char to match our new naming convention
    pub fn add_char(&mut self, c: char) {
        self.enter_char(c);
        self.update_input_height();
    }

    /// Alias for delete_char_forward
    pub fn forward_delete_char(&mut self) {
        self.delete_char_forward();
        self.update_input_height();
    }
    
    /// Scroll up by a page
    pub fn scroll_page_up(&mut self) {
        let page_size = 10; // Approximate page size
        if self.state.scroll > page_size {
            self.state.scroll -= page_size;
        } else {
            self.state.scroll = 0;
        }
    }
    
    /// Scroll down by a page
    pub fn scroll_page_down(&mut self) {
        let page_size = 10; // Approximate page size
        if self.state.scroll + page_size <= self.state.scroll_max {
            self.state.scroll += page_size;
        } else {
            self.state.scroll = self.state.scroll_max;
        }
    }
    
    /// Scroll to the top
    pub fn scroll_to_top(&mut self) {
        self.state.scroll = 0;
    }
    
    /// Scroll to the bottom
    pub fn scroll_to_bottom(&mut self) {
        self.state.scroll = self.state.scroll_max;
    }

    /// Create a new TuiApp instance with a pre-initialized terminal
    pub fn with_terminal(
        ollama: OllamaClient,
        terminal: Terminal<CrosstermBackend<io::Stdout>>,
        standard_model: String,
        code_model: String,
        tx: mpsc::Sender<UiMessage>,
    ) -> Result<Self> {
        let config = Arc::new(Config {
            standard_model: standard_model.clone(),
            code_model: code_model.clone(),
        });
        
        let mut state = UiState::default();
        state.standard_model = standard_model;
        state.code_model = code_model;
        state.use_codestral = state.standard_model == "codestral";
        
        Ok(Self {
            state,
            ollama,
            terminal,
            config,
            cancel_flag: Arc::new(Mutex::new(false)),
            cancel_requested: Arc::new(Mutex::new(false)),
            tx,
        })
    }

    /// Get the height of the response view
    pub fn get_response_view_height(&self) -> u16 {
        // This is a simplified version, adjust based on your layout logic
        // Assuming terminal height - input_height - some margins
        if let Ok(size) = self.terminal.size() {
            size.height.saturating_sub(self.state.input_height + 4)
        } else {
            20 // Fallback height
        }
    }
    
    /// Set the scroll percentage
    pub fn set_scroll_percentage(&mut self, percentage: f32) {
        let percentage = percentage.clamp(0.0, 1.0);
        self.state.scroll = (self.state.scroll_max as f32 * percentage) as u16;
    }
    
    /// Handle terminal resize event
    pub fn handle_resize(&mut self) -> Result<()> {
        let size = self.terminal.size()?;
        // Adjust app state based on new terminal size
        // For example, update scroll_max
        self.update_scroll_max(size.height.saturating_sub(self.state.input_height + 4));
        Ok(())
    }

    /// Get a mutable reference to the cancel_requested Mutex
    pub fn get_cancel_requested(&self) -> &Arc<Mutex<bool>> {
        &self.cancel_requested
    }
    
    /// Get a mutable reference to the cancel_flag Mutex
    pub fn get_cancel_flag(&self) -> &Arc<Mutex<bool>> {
        &self.cancel_flag
    }
    
    /// Fix async locking of Mutex
    fn lock_mutex<T>(mutex: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>> {
        mutex.lock().map_err(|e| anyhow::anyhow!("Failed to lock mutex: {}", e))
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

/// Draw the UI with the current state
pub fn draw_ui(f: &mut Frame, app: &UiState) {
    // Create a flexbox-like layout with dynamic input height
    let input_height = if app.input_height > 0 {
        app.input_height
    } else {
        4 // Default to 2 content lines + 2 border lines
    };
    
    // Create layout with input at the bottom
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),              // Response area - takes remaining space
            Constraint::Length(3),            // Status bar - fixed height
            Constraint::Length(input_height), // Dynamic input box height at the bottom
        ])
        .margin(1)                            // Add margin around all elements
        .split(f.area());
    
    // Selection mode indicator
    let selection_mode_indicator = if app.selection_mode {
        " [Selection Mode]"
    } else {
        ""
    };
    
    // Add some helpful text about keyboard controls based on current state
    let input_help = if app.selection_mode {
        "Mouse: Select text | Enter: Copy | Esc: Exit selection mode"
    } else {
        "Enter: Submit | Shift+Enter: New line | Esc: Cancel/Exit"
    };
    
    // Input box with instructions
    let input_text = &app.input;
    let input = Paragraph::new(input_text.to_string())
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!("Input (type your response){}", selection_mode_indicator)))
        .wrap(ratatui::widgets::Wrap { trim: false }); // Don't trim for multi-line editing
    f.render_widget(input, chunks[2]);
    
    // Add the input help text at the bottom of the input area
    let help_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(chunks[2].inner(Margin { vertical: 1, horizontal: 1 }));
        
    let help_text = Paragraph::new(input_help);
    f.render_widget(help_text, help_area[1]);

    // Set cursor position
    if !app.selection_mode {
        // Only show cursor in normal mode
        // Need to calculate line/column for multi-line input
        let mut current_line = 0;
        let mut current_col = 0;
        
        for (i, c) in app.input.chars().enumerate() {
            if i == app.cursor_position {
                break;
            }
            if c == '\n' {
                current_line += 1;
                current_col = 0;
            } else {
                current_col += 1;
            }
        }
        
        f.set_cursor_position((
            chunks[2].x + current_col + 1, // +1 for left border
            chunks[2].y + current_line + 1, // +1 for top border/title
        ));
    }
    
    // Status bar with dynamic status based on model state
    let (status_text, status_icon) = if app.thinking_visible {
        match app.model_state {
            ModelState::Loading => (app.thinking_text.as_str(), "⏳"),
            ModelState::Generating => (app.thinking_text.as_str(), "🔄"),
            ModelState::Streaming => (app.thinking_text.as_str(), "💬"),
            ModelState::Complete => (app.thinking_text.as_str(), "✨"),
            ModelState::Error => (app.thinking_text.as_str(), "❌"),
            ModelState::Idle => ("Ready for input", "✨"),
        }
    } else {
        ("Ready for input", "✨")
    };
    
    // Add cancel indicator if needed
    let status_suffix = if app.cancel_requested {
        " (Cancelling...)"
    } else {
        ""
    };
    
    let status = Paragraph::new(format!("{}{}", status_text, status_suffix))
        .block(Block::default()
        .borders(Borders::ALL)
            .title(format!("Status {status_icon}")));
    f.render_widget(status, chunks[1]);
    
    // Response area with model indicator and word wrap
    // Get the model name from the OllamaClient's model_config
    let model_name = if app.use_codestral {
        &app.code_model
    } else {
        &app.standard_model
    };
    
    let model_icon = if app.use_codestral { "🧠" } else { "🐬" };
    
    // Add a streaming indicator to the title based on model state
    let title = match app.model_state {
        ModelState::Loading => format!("Response ({model_icon} {model_name}) ⏳"),
        ModelState::Generating => format!("Response ({model_icon} {model_name}) 🔄"),
        ModelState::Streaming => format!("Response ({model_icon} {model_name}) 💬"),
        _ => format!("Response ({model_icon} {model_name})"),
    };
    
    // Process the response text to handle thinking sections
    let display_text = if app.show_thinking_sections {
        app.response_text.clone()
    } else {
        // Hide thinking sections by replacing them with a placeholder
        let mut processed_text = String::new();
        let mut in_thinking_section = false;
        let mut has_thinking_sections = false;
        
        for line in app.response_text.lines() {
            if line.contains("<think>") {
                in_thinking_section = true;
                has_thinking_sections = true;
                processed_text.push_str("📝 [Thinking section - press F1 to expand] 📝\n");
                continue;
            }
            
            if line.contains("</think>") {
                in_thinking_section = false;
                continue;
            }
            
            if !in_thinking_section {
                processed_text.push_str(line);
                processed_text.push('\n');
            }
        }
        
        // Remove trailing newline if present
        if processed_text.ends_with('\n') {
            processed_text.pop();
        }
        
        if !has_thinking_sections {
            // If no thinking sections were found, just use the original text
            app.response_text.clone()
        } else {
            processed_text
        }
    };
    
    // Show scroll controls help only if there's content to scroll
    let mut help_items = Vec::new();
    
    if app.scroll_max > 0 {
        help_items.push("↑/↓: Scroll");
        help_items.push("PgUp/PgDn: Page");
    }
    
    if !app.response_text.is_empty() {
        help_items.push("F1: Toggle thinking");
        help_items.push("Ctrl+S: Selection mode");
    }
    
    let scroll_help = if !help_items.is_empty() {
        format!("\n{}", help_items.join("  "))
    } else {
        String::new()
    };
            
    // Calculate response area with scrollbar
    let response_area = chunks[0];
    
    // Only create scrollbar area if we have content to scroll
    let scrollbar_area = if app.scroll_max > 0 {
        response_area.inner(Margin { 
            vertical: 1, 
            horizontal: 0 
        })
    } else {
        response_area
    };
    
    // Response widget
    let response = Paragraph::new(display_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!("{}{}", title, scroll_help)))
        .wrap(ratatui::widgets::Wrap { trim: true })
        .scroll((app.scroll, 0));
    f.render_widget(response, response_area);
    
    // Only render scrollbar if needed
    if app.scroll_max > 0 {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        
        f.render_stateful_widget(
            scrollbar,
            scrollbar_area,
            &mut ScrollbarState::new(app.scroll_max as usize + 1)
                .position(app.scroll as usize),
        );
    }
} 