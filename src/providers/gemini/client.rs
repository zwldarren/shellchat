use crate::core::error::SchatError;
use crate::providers::base_client::HttpClient;
use crate::providers::gemini::types::*;
use std::collections::HashMap;

/// Parser for Gemini streaming responses
pub fn gemini_stream_parser(data: String) -> Result<Option<String>, SchatError> {
    let mut content = String::new();

    // Gemini stream sends JSON chunks directly, sometimes multiple per response
    for line in data.lines().filter(|l| !l.trim().is_empty()) {
        let json_str = if line.starts_with("data: ") {
            line[6..].trim()
        } else {
            line.trim()
        };

        if json_str.is_empty() {
            continue;
        }

        if json_str == "[" || json_str == "]" {
            continue;
        }

        let clean_json = if json_str.starts_with(',') {
            &json_str[1..]
        } else {
            json_str
        };

        if clean_json.is_empty() {
            continue;
        }

        let parsed: serde_json::Value = serde_json::from_str(clean_json).map_err(|e| {
            SchatError::Serialization(format!(
                "Failed to parse stream data: {}. Data: '{}'",
                e, clean_json
            ))
        })?;

        if let Some(candidates) = parsed.get("candidates").and_then(|c| c.as_array()) {
            if let Some(first_candidate) = candidates.first() {
                if let Some(content_node) = first_candidate.get("content") {
                    if let Some(parts) = content_node.get("parts").and_then(|p| p.as_array()) {
                        if let Some(first_part) = parts.first() {
                            if let Some(text) = first_part.get("text").and_then(|t| t.as_str()) {
                                content.push_str(text);
                            }
                        }
                    }
                }
            }
        }
    }

    if content.is_empty() {
        Ok(None)
    } else {
        Ok(Some(content))
    }
}

#[derive(Clone)]
pub struct GeminiClient {
    pub model: String,
    client: HttpClient,
}

impl GeminiClient {
    pub fn new(
        base_url: String,
        api_key: String,
        model: String,
        extra_headers: Option<HashMap<String, String>>,
    ) -> Self {
        let mut client = HttpClient::new(base_url, None, extra_headers);

        // Add API key to query params
        client.add_query_param("key", api_key);

        Self { client, model }
    }

    pub async fn generate_content(
        &self,
        messages: &[crate::providers::Message],
    ) -> Result<String, SchatError> {
        let payload = self.build_payload(messages)?;
        let response = self
            .client
            .post(
                &format!("v1beta/models/{}:generateContent", self.model),
                &payload,
            )
            .await?;

        let response_body: String = response.text().await?;
        let parsed: GeminiResponse = serde_json::from_str(&response_body).map_err(|e| {
            SchatError::Serialization(format!("Failed to parse Gemini response: {}", e))
        })?;

        if let Some(candidate) = parsed.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                return Ok(part.text.clone());
            }
        }

        Err(SchatError::Api("No valid response from Gemini".to_string()))
    }

    pub async fn generate_content_stream(
        &self,
        messages: &[crate::providers::Message],
    ) -> Result<futures::stream::BoxStream<'static, Result<String, SchatError>>, SchatError> {
        let payload = self.build_payload(messages)?;
        let mut client = self.client.clone();
        client.add_query_param("alt", "sse".to_string());
        let response = client
            .post(
                &format!("v1beta/models/{}:streamGenerateContent", self.model),
                &payload,
            )
            .await?;

        let stream = client
            .stream_response(response, gemini_stream_parser)
            .await?;

        Ok(stream)
    }

    fn build_payload(
        &self,
        messages: &[crate::providers::Message],
    ) -> Result<GeminiRequest, SchatError> {
        let mut contents = Vec::new();
        let mut system_instruction = None;

        // Separate system messages from conversation messages
        let mut conversation_messages = Vec::new();
        for message in messages {
            if message.role == crate::providers::Role::System {
                // Collect all system messages
                if system_instruction.is_none() {
                    system_instruction = Some(SystemInstruction {
                        parts: vec![GeminiPart {
                            text: message.content.clone(),
                        }],
                    });
                }
            } else {
                conversation_messages.push(message);
            }
        }

        // Process conversation messages
        for message in conversation_messages {
            let role = match message.role {
                crate::providers::Role::User => "user",
                crate::providers::Role::Assistant => "model",
                _ => continue,
            }
            .to_string();

            contents.push(GeminiContentPart {
                role,
                parts: vec![GeminiPart {
                    text: message.content.clone(),
                }],
            });
        }

        Ok(GeminiRequest {
            contents,
            system_instruction,
        })
    }
}
