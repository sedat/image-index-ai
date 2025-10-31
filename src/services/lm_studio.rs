use std::env;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, info};

use crate::tagging::parse_tags;

const IMAGE_TAGGING_PROMPT: &str = r#"
You are an image tagging assistant. Your task is to analyze the given image and generate a comma-separated list of relevant tags or keywords that can be used to categorize and search for similar images in a database.

When generating tags, please follow these guidelines:

1. Use concise, descriptive words or short phrases that accurately describe the content of the image.
2. Avoid using full sentences or unnecessary words in the tags.
3. Include tags that describe the main subject(s), objects, scenes, activities, emotions, colors, and any other relevant aspects of the image.
4. Use plural forms for nouns when appropriate (e.g., "trees" instead of "tree").
5. Separate each tag with a comma and a space (e.g., "nature, landscape, trees, mountain").
6. Do not include any additional text or explanations beyond the comma-separated list of tags.

Please analyze the provided image and generate a list of relevant tags following the guidelines above.
"#;

const SEARCH_TAGGING_PROMPT: &str = "You are a photo tagging assistant. Extract concise, comma-separated tags from the user's search query so they can be matched against stored photo metadata.";

#[derive(Clone)]
pub struct LmStudioClient {
    http: Client,
    base_url: String,
    image_model: String,
    text_model: String,
    embed_model: String,
    temperature: f32,
}

impl LmStudioClient {
    pub fn new(http: Client) -> Self {
        let base_url = env::var("LMSTUDIO_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:1234/v1".to_string());
        let image_model =
            env::var("LMSTUDIO_IMAGE_MODEL").unwrap_or_else(|_| "qwen/qwen3-vl-4b".to_string());
        let text_model =
            env::var("LMSTUDIO_TEXT_MODEL").unwrap_or_else(|_| "qwen/qwen3-vl-4b".to_string());
        let embed_model_raw = env::var("LMSTUDIO_EMBED_MODEL")
            .unwrap_or_else(|_| "text-embedding-nomic-embed-text-v1.5".to_string());
        let embed_model = if embed_model_raw == "nomic-ai/nomic-embed-text-v1.5" {
            // Map LM Studio's internal name to the OpenAI-compatible alias we prefer.
            "text-embedding-nomic-embed-text-v1.5".to_string()
        } else {
            embed_model_raw
        };
        let temperature = env::var("LMSTUDIO_TEMPERATURE")
            .ok()
            .and_then(|value| value.parse::<f32>().ok())
            .unwrap_or(0.2);

        info!(
            base_url = %base_url,
            image_model = %image_model,
            text_model = %text_model,
            embed_model = %embed_model,
            "configured LM Studio client"
        );

        Self {
            http,
            base_url,
            image_model,
            text_model,
            embed_model,
            temperature,
        }
    }

    pub async fn tag_image(&self, base64_image: &str, mime_type: &str) -> Result<Vec<String>> {
        let image_url = format!("data:{};base64,{}", mime_type, base64_image);

        let messages = vec![
            json!({
                "role": "system",
                "content": [{
                    "type": "text",
                    "text": format!(
                        "{} {}",
                        IMAGE_TAGGING_PROMPT.trim(),
                        "Respond only with comma-separated tags."
                    ),
                }],
            }),
            json!({
                "role": "user",
                "content": [
                    {"type": "text", "text": "Analyze this image and return the tags."},
                    {"type": "image_url", "image_url": {"url": image_url}},
                ],
            }),
        ];

        info!(
            model = %self.image_model,
            mime_type,
            "requesting image tags from LM Studio"
        );

        let response = self
            .chat_completion(&self.image_model, messages)
            .await
            .context("LM Studio failed to tag image")?;

        let tags = parse_tags(&response);

        info!(
            model = %self.image_model,
            tag_count = tags.len(),
            "received image tags from LM Studio"
        );

        Ok(tags)
    }

    pub async fn tags_from_query(&self, query: &str) -> Result<Vec<String>> {
        let messages = vec![
            json!({
                "role": "system",
                "content": [{
                    "type": "text",
                    "text": format!(
                        "{} {}",
                        SEARCH_TAGGING_PROMPT,
                        "Only respond with comma-separated tags."
                    ),
                }],
            }),
            json!({
                "role": "user",
                "content": [{"type": "text", "text": query}],
            }),
        ];

        info!(
            model = %self.text_model,
            "requesting search tags from LM Studio"
        );

        let response = self
            .chat_completion(&self.text_model, messages)
            .await
            .context("LM Studio failed to process search query")?;

        let tags = parse_tags(&response);

        info!(
            model = %self.text_model,
            tag_count = tags.len(),
            "received search tags from LM Studio"
        );

        Ok(tags)
    }

    async fn chat_completion(&self, model: &str, messages: Vec<Value>) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let body = json!({
            "model": model,
            "messages": messages,
            "temperature": self.temperature,
        });

        debug!(model = %model, url = %url, "sending LM Studio chat completion request");

        let response = self
            .http
            .post(url)
            .json(&body)
            .send()
            .await
            .context("failed to contact LM Studio")?
            .error_for_status()
            .context("LM Studio returned an error status")?;

        let payload: ChatCompletionResponse = response
            .json()
            .await
            .context("LM Studio response was not valid JSON")?;

        let choice = payload
            .choices
            .into_iter()
            .next()
            .context("LM Studio response contained no choices")?;

        let text = choice
            .message
            .content
            .into_string()
            .context("LM Studio response did not include textual content")?;

        let trimmed = text.trim().to_string();

        debug!(
            model = %model,
            response_len = trimmed.len(),
            "received LM Studio chat completion response"
        );

        Ok(trimmed)
    }

    pub async fn embed_texts(&self, inputs: &[String]) -> Result<Vec<Vec<f32>>> {
        let url = format!("{}/embeddings", self.base_url.trim_end_matches('/'));

        let body = json!({
            "model": self.embed_model,
            "input": inputs,
        });

        info!(
            model = %self.embed_model,
            count = inputs.len(),
            "requesting embeddings from LM Studio"
        );

        let response = self
            .http
            .post(url)
            .json(&body)
            .send()
            .await
            .context("failed to contact LM Studio for embeddings")?
            .error_for_status()
            .context("LM Studio embeddings returned an error status")?;

        let payload: EmbeddingsResponse = response
            .json()
            .await
            .context("LM Studio embeddings response was not valid JSON")?;

        let mut result = Vec::with_capacity(payload.data.len());
        for item in payload.data {
            result.push(item.embedding);
        }

        if result.len() != inputs.len() {
            anyhow::bail!(
                "LM Studio returned {} embeddings for {} inputs",
                result.len(),
                inputs.len()
            );
        }

        Ok(result)
    }
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: MessageContent,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum MessageContent {
    Text(String),
    Parts(Vec<MessagePart>),
}

impl MessageContent {
    fn into_string(self) -> Option<String> {
        match self {
            MessageContent::Text(text) => Some(text),
            MessageContent::Parts(parts) => {
                let text = parts
                    .into_iter()
                    .filter_map(|part| part.text)
                    .map(|segment| segment.trim().to_string())
                    .filter(|segment| !segment.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");

                if text.is_empty() {
                    None
                } else {
                    Some(text)
                }
            }
        }
    }
}

#[derive(Deserialize)]
struct MessagePart {
    #[serde(rename = "type")]
    _kind: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct EmbeddingsResponse {
    data: Vec<EmbeddingItem>,
}

#[derive(Deserialize)]
struct EmbeddingItem {
    embedding: Vec<f32>,
}
