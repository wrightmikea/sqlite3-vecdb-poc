//! Ollama API client for embedding generation
//!
//! Provides a client to interact with a local Ollama instance for generating
//! text embeddings using various models.

use crate::error::{Result, VectDbError};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// Ollama API client
#[derive(Clone)]
pub struct OllamaClient {
    base_url: String,
    client: Client,
    timeout: Duration,
}

impl OllamaClient {
    /// Create a new Ollama client
    pub fn new(base_url: String, timeout_seconds: u64) -> Result<Self> {
        let timeout = Duration::from_secs(timeout_seconds);

        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| VectDbError::Http(e))?;

        info!("Created Ollama client with base URL: {}", base_url);

        Ok(Self {
            base_url,
            client,
            timeout,
        })
    }

    /// Check if Ollama service is available
    pub async fn health_check(&self) -> Result<bool> {
        debug!("Performing health check on Ollama");

        let url = format!("{}/api/tags", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => {
                let is_ok = response.status().is_success();
                if is_ok {
                    info!("Ollama health check passed");
                } else {
                    warn!("Ollama health check failed with status: {}", response.status());
                }
                Ok(is_ok)
            }
            Err(e) => {
                warn!("Ollama health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Generate an embedding for a single text
    pub async fn embed(&self, model: &str, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed_batch(model, &[text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| VectDbError::EmbeddingFailed("No embedding returned".to_string()))
    }

    /// Generate embeddings for a batch of texts with retry logic
    pub async fn embed_batch(&self, model: &str, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        debug!("Generating embeddings for {} texts using model {}", texts.len(), model);

        let url = format!("{}/api/embeddings", self.base_url);

        let mut embeddings = Vec::with_capacity(texts.len());

        // Process texts one at a time (Ollama's embeddings endpoint takes one input at a time)
        for (idx, text) in texts.iter().enumerate() {
            let request = EmbedRequest {
                model: model.to_string(),
                prompt: text.clone(),
            };

            // Retry logic with exponential backoff
            let embedding = self.embed_with_retry(&url, &request).await?;
            embeddings.push(embedding);

            if (idx + 1) % 10 == 0 {
                debug!("Generated {}/{} embeddings", idx + 1, texts.len());
            }
        }

        info!("Successfully generated {} embeddings", embeddings.len());

        Ok(embeddings)
    }

    /// Generate a single embedding with retry logic
    async fn embed_with_retry(&self, url: &str, request: &EmbedRequest) -> Result<Vec<f32>> {
        const MAX_RETRIES: u32 = 3;
        const INITIAL_BACKOFF_MS: u64 = 100;

        let mut retries = 0;
        let mut backoff_ms = INITIAL_BACKOFF_MS;

        loop {
            match self.client.post(url).json(request).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let embed_response: EmbedResponse = response.json().await.map_err(|e| {
                            VectDbError::EmbeddingFailed(format!("Failed to parse response: {}", e))
                        })?;
                        return Ok(embed_response.embedding);
                    } else if response.status().as_u16() == 404 {
                        // Model not found - no point in retrying
                        let error_text = response.text().await.unwrap_or_else(|_| "Model not found".to_string());
                        return Err(VectDbError::EmbeddingFailed(format!(
                            "Model '{}' not found. {}",
                            request.model, error_text
                        )));
                    } else {
                        // Server error - may be transient
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());

                        if retries < MAX_RETRIES {
                            warn!(
                                "Embedding request failed with status {} (attempt {}/{}): {}",
                                status,
                                retries + 1,
                                MAX_RETRIES + 1,
                                error_text
                            );
                            retries += 1;
                            sleep(Duration::from_millis(backoff_ms)).await;
                            backoff_ms *= 2; // Exponential backoff
                            continue;
                        } else {
                            return Err(VectDbError::EmbeddingFailed(format!(
                                "Ollama API returned error {} after {} retries: {}",
                                status, MAX_RETRIES, error_text
                            )));
                        }
                    }
                }
                Err(e) => {
                    // Network error - may be transient
                    if retries < MAX_RETRIES {
                        warn!(
                            "Network error during embedding request (attempt {}/{}): {}",
                            retries + 1,
                            MAX_RETRIES + 1,
                            e
                        );
                        retries += 1;
                        sleep(Duration::from_millis(backoff_ms)).await;
                        backoff_ms *= 2;
                        continue;
                    } else {
                        return Err(VectDbError::OllamaUnavailable(format!(
                            "Failed to connect to Ollama after {} retries: {}",
                            MAX_RETRIES, e
                        )));
                    }
                }
            }
        }
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        debug!("Listing available models from Ollama");

        let url = format!("{}/api/tags", self.base_url);

        let response = self.client.get(&url).send().await.map_err(|e| {
            VectDbError::OllamaUnavailable(format!("Failed to connect to Ollama: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(VectDbError::OllamaUnavailable(format!(
                "Ollama API returned error: {}",
                response.status()
            )));
        }

        let tags_response: TagsResponse = response.json().await.map_err(|e| {
            VectDbError::OllamaUnavailable(format!("Failed to parse response: {}", e))
        })?;

        let models: Vec<ModelInfo> = tags_response
            .models
            .into_iter()
            .map(|m| ModelInfo {
                name: m.name,
                size: m.size,
                modified_at: m.modified_at,
            })
            .collect();

        info!("Found {} models", models.len());

        Ok(models)
    }

    /// Check if a specific model is available
    /// Handles both "model" and "model:tag" formats
    pub async fn has_model(&self, model_name: &str) -> Result<bool> {
        let models = self.list_models().await?;

        // Check for exact match first
        if models.iter().any(|m| m.name == model_name) {
            return Ok(true);
        }

        // If model_name doesn't have a tag, try matching with :latest
        if !model_name.contains(':') {
            let with_latest = format!("{}:latest", model_name);
            if models.iter().any(|m| m.name == with_latest) {
                return Ok(true);
            }
        }

        // Try partial matching (model name without tag)
        let base_name = model_name.split(':').next().unwrap_or(model_name);
        Ok(models.iter().any(|m| {
            let model_base = m.name.split(':').next().unwrap_or(&m.name);
            model_base == base_name
        }))
    }

    /// Get information about the client configuration
    pub fn info(&self) -> ClientInfo {
        ClientInfo {
            base_url: self.base_url.clone(),
            timeout_seconds: self.timeout.as_secs(),
        }
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Serialize)]
struct EmbedRequest {
    model: String,
    prompt: String,
}

#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embedding: Vec<f32>,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<ModelDetail>,
}

#[derive(Debug, Deserialize)]
struct ModelDetail {
    name: String,
    #[serde(default)]
    size: u64,
    modified_at: String,
}

/// Information about an available model
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
}

/// Information about the Ollama client configuration
#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub base_url: String,
    pub timeout_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = OllamaClient::new("http://localhost:11434".to_string(), 30).unwrap();
        let info = client.info();

        assert_eq!(info.base_url, "http://localhost:11434");
        assert_eq!(info.timeout_seconds, 30);
    }

    #[tokio::test]
    async fn test_health_check() {
        let client = OllamaClient::new("http://localhost:11434".to_string(), 5).unwrap();

        // This will fail if Ollama is not running, which is expected in CI
        let result = client.health_check().await;

        // We just check that the method executes without panicking
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_models() {
        let client = OllamaClient::new("http://localhost:11434".to_string(), 5).unwrap();

        // This will fail if Ollama is not running, which is expected in CI
        let result = client.list_models().await;

        // Check the method completes (may return error if Ollama not running)
        match result {
            Ok(models) => {
                println!("Found {} models", models.len());
            }
            Err(e) => {
                println!("Expected error (Ollama likely not running): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_embed_batch_empty() {
        let client = OllamaClient::new("http://localhost:11434".to_string(), 5).unwrap();

        let result = client.embed_batch("test-model", &[]).await.unwrap();
        assert_eq!(result.len(), 0);
    }
}
