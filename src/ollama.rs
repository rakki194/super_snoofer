use anyhow::Result;
use ollama_rs::{
    generation::completion::request::GenerationRequest,
    Ollama,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub const DOLPHIN_MODEL: &str = "cognitivecomputations_Dolphin3.0-R1-Mistral-24B-Q5_K_M:latest";
pub const CODESTRAL_MODEL: &str = "codestral:latest";

#[derive(Clone)]
pub struct OllamaClient {
    client: Arc<Mutex<Ollama>>,
}

impl std::fmt::Debug for OllamaClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OllamaClient")
            .field("client", &"Arc<Mutex<Ollama>>")
            .finish()
    }
}

impl OllamaClient {
    pub async fn new() -> Result<Self> {
        let ollama = Ollama::default();
        Ok(Self {
            client: Arc::new(Mutex::new(ollama)),
        })
    }

    pub async fn generate_response(&self, prompt: &str, use_codestral: bool) -> Result<String> {
        let model = if use_codestral {
            CODESTRAL_MODEL
        } else {
            DOLPHIN_MODEL
        };

        let client = self.client.lock().await;
        let response = client
            .generate(GenerationRequest::new(
                model.to_string(),
                prompt.to_string(),
            ))
            .await?;

        Ok(response.response)
    }
}

impl Default for OllamaClient {
    fn default() -> Self {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(Self::new())
            .unwrap()
    }
} 