#![warn(clippy::all, clippy::pedantic)]

#[cfg(test)]
mod tests {
    use crate::{draw_ui, OllamaClient, TuiApp};

    use super::*;
    use ratatui::{
        backend::TestBackend,
        Terminal,
    };

    #[tokio::test]
    async fn test_tui_app_new() {
        let mut app = TuiApp::new(OllamaClient::new(), false).unwrap();
        assert_eq!(app.input, "");
        assert_eq!(app.cursor_position, 0);
        assert_eq!(app.scroll, 0);
        assert!(app.thinking_visible);
        assert_eq!(app.thinking_text, "");
        assert_eq!(app.response_text, "");
        assert!(app.last_response.is_none());
        assert!(!app.use_codestral);
    }

    #[tokio::test]
    async fn test_input_handling() {
        let mut app = TuiApp::new(OllamaClient::new(), false).unwrap();

        // Test entering characters
        app.enter_char('h');
        app.enter_char('i');
        assert_eq!(app.input, "hi");
        assert_eq!(app.cursor_position, 2);

        // Test cursor movement
        app.move_cursor_left();
        assert_eq!(app.cursor_position, 1);
        app.move_cursor_right();
        assert_eq!(app.cursor_position, 2);

        // Test backspace
        app.delete_char();
        assert_eq!(app.input, "h");
        assert_eq!(app.cursor_position, 1);

        app.delete_char_forward();
        assert_eq!(app.input, "h");
        assert_eq!(app.cursor_position, 1);
    }

    #[tokio::test]
    async fn test_draw_ui() {
        let backend = TestBackend::new(20, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = TuiApp::new(OllamaClient::new(), false).unwrap();

        terminal.draw(|f| {
            draw_ui(
                f,
                &app.input,
                app.cursor_position,
                app.thinking_visible,
                &app.thinking_text,
                &app.response_text,
            )
        }).unwrap();
    }

    #[tokio::test]
    async fn test_codestral_flag() {
        // Test default (Dolphin)
        let app = TuiApp::new(OllamaClient::new(), false).unwrap();
        assert!(!app.use_codestral);

        // Test Codestral
        let app = TuiApp::new(OllamaClient::new(), true).unwrap();
        assert!(app.use_codestral);
    }

    #[tokio::test]
    async fn test_thinking_states() {
        let mut app = TuiApp::new(OllamaClient::new(), false).unwrap();

        // Initial state
        assert!(app.thinking_visible);
        assert_eq!(app.thinking_text, "");

        // Starting state
        app.thinking_text = "🤔 Starting...".to_string();
        assert_eq!(app.thinking_text, "🤔 Starting...");

        // Streaming state
        app.thinking_text = "💭 Streaming response...".to_string();
        assert_eq!(app.thinking_text, "💭 Streaming response...");

        // Done state
        app.thinking_text = "✨ Done!".to_string();
        assert_eq!(app.thinking_text, "✨ Done!");

        // Toggle thinking visibility
        app.toggle_thinking();
        assert!(!app.thinking_visible);
        app.toggle_thinking();
        assert!(app.thinking_visible);
    }
} 