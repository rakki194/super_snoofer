#![warn(clippy::all, clippy::pedantic)]

use anyhow::Result;
use futures::StreamExt;
use ollama_rs::{
    generation::completion::request::GenerationRequest,
    Ollama,
};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Default model for standard queries
pub const DEFAULT_MODEL: &str = "cognitivecomputations_Dolphin3.0-R1-Mistral-24B-Q5_K_M:latest";
/// Default code model for code-focused queries
pub const DEFAULT_CODE_MODEL: &str = "codestral:latest";

/// Configuration for Ollama models
#[derive(Debug, Clone)]
pub struct ModelConfig {
    /// Model to use for standard queries
    pub standard_model: String,
    /// Model to use for code-focused queries
    pub code_model: String,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            standard_model: DEFAULT_MODEL.to_string(),
            code_model: DEFAULT_CODE_MODEL.to_string(),
        }
    }
}

impl ModelConfig {
    /// Create a new configuration with custom model names
    #[must_use] pub fn new(standard_model: String, code_model: String) -> Self {
        Self {
            standard_model,
            code_model,
        }
    }
    
    /// Get the appropriate model based on the code flag
    #[must_use] pub fn get_model(&self, use_code_model: bool) -> &str {
        if use_code_model {
            &self.code_model
        } else {
            &self.standard_model
        }
    }
}

#[derive(Clone)]
pub struct OllamaClient {
    client: Arc<Mutex<Ollama>>,
    pub model_config: ModelConfig,
}

impl std::fmt::Debug for OllamaClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OllamaClient")
            .field("client", &"Arc<Mutex<Ollama>>")
            .field("model_config", &self.model_config)
            .finish()
    }
}

impl OllamaClient {
    /// Creates a new `OllamaClient` with default configuration
    #[must_use]
    pub fn new() -> Self {
        let ollama = Ollama::default();
        Self {
            client: Arc::new(Mutex::new(ollama)),
            model_config: ModelConfig::default(),
        }
    }
    
    /// Create a new client with custom model configuration
    #[must_use]
    pub fn with_config(model_config: ModelConfig) -> Self {
        let ollama = Ollama::default();
        Self {
            client: Arc::new(Mutex::new(ollama)),
            model_config,
        }
    }

    /// Generate a response using Ollama
    /// 
    /// # Errors
    /// Returns an error if the response generation fails due to Ollama API issues or network problems
    pub async fn generate_response(&self, prompt: &str, use_code_model: bool) -> Result<String> {
        let model = self.model_config.get_model(use_code_model);

        let client = self.client.lock().await;
        let response = client
            .generate(GenerationRequest::new(
                model.to_string(),
                prompt.to_string(),
            ))
            .await?;

        Ok(response.response)
    }
    
    /// Stream a response using Ollama's API by implementing our own streaming solution
    /// 
    /// # Errors
    /// Returns an error if streaming fails due to Ollama API issues or network problems
    pub async fn stream_response(&self, prompt: &str, use_code_model: bool, tx: mpsc::Sender<String>) -> Result<()> {
        let model = self.model_config.get_model(use_code_model);
        
        // We won't use the official client's request type directly
        // since we need to set stream=true
        
        let client = self.client.lock().await;
        let url = format!("{}api/generate", client.url_str());
        
        // Release the mutex lock before making HTTP requests
        drop(client);
        
        // Create a regular reqwest client for streaming
        let client = reqwest::Client::new();
        
        // Create our own JSON payload with stream set to true
        let json_payload = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": true
        });
        
        let serialized = serde_json::to_string(&json_payload)?;
        
        // Send request
        let res = client.post(url)
            .header("Content-Type", "application/json")
            .body(serialized)
            .send()
            .await?;
            
        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(anyhow::anyhow!("Request failed: {}", error_text));
        }
        
        // Process the streaming response
        let mut stream = res.bytes_stream();
        let mut buffer = String::new();
        
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            let chunk_str = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_str);
            
            // Process the buffer line by line
            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].to_string();
                let remainder = buffer[pos + 1..].to_string();
                buffer = remainder;
                
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                // Parse the JSON response
                if let Ok(response) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(text) = response.get("response").and_then(|v| v.as_str()) {
                        if !text.is_empty() {
                            let _ = tx.send(text.to_string()).await;
                        }
                    }
                }
            }
        }
        
        // Process any remaining data in the buffer
        if !buffer.is_empty() {
            if let Ok(response) = serde_json::from_str::<serde_json::Value>(&buffer) {
                if let Some(text) = response.get("response").and_then(|v| v.as_str()) {
                    if !text.is_empty() {
                        let _ = tx.send(text.to_string()).await;
                    }
                }
            }
        }
        
        Ok(())
    }
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new()
    }
} 