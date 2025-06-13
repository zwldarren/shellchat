use crate::error::SchatError;
use futures::stream::{BoxStream, StreamExt};
use reqwest::{Client, Response};
use serde::Serialize;
use std::collections::HashMap;

pub struct BaseApiClient {
    endpoint: String,
    api_key: String,
    extra_headers: HashMap<String, String>,
}

impl BaseApiClient {
    pub fn new(
        endpoint: String,
        api_key: String,
        extra_headers: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            endpoint,
            api_key,
            extra_headers: extra_headers.unwrap_or_default(),
        }
    }

    pub async fn send_request<T: Serialize + ?Sized>(
        &self,
        path: &str,
        payload: &T,
    ) -> Result<Response, SchatError> {
        // Validate API key
        if self.api_key.is_empty() {
            return Err(SchatError::Api(
                "API key is missing. Please set your API key in the configuration".to_string(),
            ));
        }

        let client = Client::builder()
            .build()
            .map_err(|e| SchatError::Network(format!("Failed to create HTTP client: {}", e)))?;

        let url = format!("{}/{}", self.endpoint, path);

        let mut request = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");

        for (key, value) in &self.extra_headers {
            request = request.header(key, value);
        }

        let response = request
            .json(payload)
            .send()
            .await
            .map_err(|e| SchatError::Network(format!("API request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());
            return Err(SchatError::Api(format!("API error: {} - {}", status, body)));
        }

        Ok(response)
    }

    pub async fn get_response_stream(
        &self,
        path: &str,
        payload: &impl Serialize,
    ) -> Result<BoxStream<'static, Result<String, SchatError>>, SchatError> {
        let response = self.send_request(path, payload).await?;
        let stream = response.bytes_stream();

        let s = stream
            .map(|item| {
                item.map_err(|e| SchatError::Network(format!("Stream error: {}", e)))
                    .and_then(|chunk| {
                        String::from_utf8(chunk.to_vec()).map_err(|e| {
                            SchatError::Serialization(format!("UTF-8 conversion error: {}", e))
                        })
                    })
            })
            .filter_map(|res| async move {
                match res {
                    Ok(s) => {
                        let mut content = String::new();
                        for line in s.lines() {
                            if line.starts_with("data:") {
                                let data = line[5..].trim();
                                if data == "[DONE]" {
                                    return None;
                                }
                                match serde_json::from_str::<StreamResponse>(data) {
                                    Ok(parsed) => {
                                        if let Some(choice) = parsed.choices.get(0) {
                                            if let Some(c) = &choice.delta.content {
                                                content.push_str(c);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        return Some(Err(SchatError::Serialization(format!(
                                            "Failed to parse stream data: {}",
                                            e
                                        ))));
                                    }
                                }
                            }
                        }
                        if content.is_empty() {
                            None
                        } else {
                            Some(Ok(content))
                        }
                    }
                    Err(e) => Some(Err(e)),
                }
            });

        Ok(s.boxed())
    }
}

#[derive(serde::Deserialize)]
struct StreamResponse {
    choices: Vec<StreamChoice>,
}

#[derive(serde::Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(serde::Deserialize)]
struct StreamDelta {
    content: Option<String>,
}
