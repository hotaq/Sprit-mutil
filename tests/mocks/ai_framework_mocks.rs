//! Mock implementations for AI framework APIs

use mockall::mock;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Mock structures for AI framework responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeResponse {
    pub content: Vec<ClaudeContent>,
    pub usage: ClaudeUsage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeContent {
    pub content_type: String,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodexResponse {
    pub choices: Vec<CodexChoice>,
    pub usage: CodexUsage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodexChoice {
    pub text: String,
    pub index: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodexUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DroidResponse {
    pub response: String,
    pub metadata: DroidMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DroidMetadata {
    pub model: String,
    pub processing_time: u64,
}

// Mock AI client trait
mock! {
    pub AiClient {}

    impl AiClientTrait for AiClient {
        async fn ask_claude(&self, prompt: &str, model: &str) -> Result<ClaudeResponse, AiError>;
        async fn ask_codex(&self, prompt: &str, model: &str) -> Result<CodexResponse, AiError>;
        async fn ask_droid(&self, prompt: &str, model: &str) -> Result<DroidResponse, AiError>;
        async fn validate_api_key(&self, api_key: &str, service: &str) -> Result<bool, AiError>;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("API request failed: {0}")]
    ApiError(String),
    #[error("Authentication failed")]
    AuthError,
    #[error("Rate limit exceeded")]
    RateLimitError,
    #[error("Network error: {0}")]
    NetworkError(String),
}

pub trait AiClientTrait {
    async fn ask_claude(&self, prompt: &str, model: &str) -> Result<ClaudeResponse, AiError>;
    async fn ask_codex(&self, prompt: &str, model: &str) -> Result<CodexResponse, AiError>;
    async fn ask_droid(&self, prompt: &str, model: &str) -> Result<DroidResponse, AiError>;
    async fn validate_api_key(&self, api_key: &str, service: &str) -> Result<bool, AiError>;
}

// Mock implementations for testing
impl MockAiClient {
    pub fn new_success_mock() -> Self {
        let mut mock = MockAiClient::new();

        // Claude API mock
        mock.expect_ask_claude()
            .returning(|_prompt, _model| {
                Ok(ClaudeResponse {
                    content: vec![ClaudeContent {
                        content_type: "text".to_string(),
                        text: "This is a mock Claude response for testing purposes.".to_string(),
                    }],
                    usage: ClaudeUsage {
                        input_tokens: 10,
                        output_tokens: 15,
                    },
                })
            });

        // Codex API mock
        mock.expect_ask_codex()
            .returning(|_prompt, _model| {
                Ok(CodexResponse {
                    choices: vec![CodexChoice {
                        text: "function mockImplementation() { return 'mock'; }".to_string(),
                        index: 0,
                    }],
                    usage: CodexUsage {
                        prompt_tokens: 8,
                        completion_tokens: 12,
                    },
                })
            });

        // Droid API mock
        mock.expect_ask_droid()
            .returning(|_prompt, _model| {
                Ok(DroidResponse {
                    response: "Mock Droid response for testing".to_string(),
                    metadata: DroidMetadata {
                        model: "droid-mock".to_string(),
                        processing_time: 100,
                    },
                })
            });

        // API key validation mock
        mock.expect_validate_api_key()
            .returning(|_api_key, _service| Ok(true));

        mock
    }

    pub fn new_error_mock() -> Self {
        let mut mock = MockAiClient::new();

        mock.expect_ask_claude()
            .returning(|_prompt, _model| Err(AiError::ApiError("Mock API error".to_string())));

        mock.expect_ask_codex()
            .returning(|_prompt, _model| Err(AiError::RateLimitError));

        mock.expect_ask_droid()
            .returning(|_prompt, _model| Err(AiError::NetworkError("Mock network error".to_string())));

        mock.expect_validate_api_key()
            .returning(|_api_key, _service| Err(AiError::AuthError));

        mock
    }
}