#![warn(clippy::all, clippy::pedantic)]

#[cfg(test)]
mod ollama_tests {
    use crate::ollama::{OllamaClient, DOLPHIN_MODEL, CODESTRAL_MODEL};
    use tokio::runtime::Runtime;

    #[test]
    fn test_ollama_client_creation() {
        let _client = OllamaClient::new();
        // Just verify we can create a client without panicking
    }

    #[test]
    fn test_ollama_client_model_selection() {
        let rt = Runtime::new().unwrap();
        let client = OllamaClient::new();

        // Test Dolphin model selection
        rt.block_on(async {
            let request = client.generate_response("test", false).await;
            assert!(request.is_ok());
            let response = request.unwrap();
            assert!(!response.is_empty(), "Response should not be empty");
        });

        // Test Codestral model selection
        rt.block_on(async {
            let request = client.generate_response("test", true).await;
            assert!(request.is_ok());
            let response = request.unwrap();
            assert!(!response.is_empty(), "Response should not be empty");
        });
    }

    #[test]
    fn test_ollama_client_error_handling() {
        let rt = Runtime::new().unwrap();
        let client = OllamaClient::new();

        // Test with empty prompt
        rt.block_on(async {
            let request = client.generate_response("", false).await;
            assert!(request.is_ok());
            let response = request.unwrap();
            assert!(!response.is_empty(), "Response should not be empty even with empty prompt");
        });

        // Test with very long prompt
        rt.block_on(async {
            let long_prompt = "x".repeat(10000);
            let request = client.generate_response(&long_prompt, false).await;
            assert!(request.is_ok());
            let response = request.unwrap();
            assert!(!response.is_empty(), "Response should not be empty with long prompt");
        });
    }

    #[test]
    fn test_ollama_model_constants() {
        assert!(!DOLPHIN_MODEL.is_empty());
        assert!(!CODESTRAL_MODEL.is_empty());
        assert_ne!(DOLPHIN_MODEL, CODESTRAL_MODEL);
        assert!(DOLPHIN_MODEL.contains("Dolphin"), "Dolphin model name should contain 'Dolphin'");
        assert!(CODESTRAL_MODEL.contains("codestral"), "Codestral model name should contain 'codestral'");
    }
} 