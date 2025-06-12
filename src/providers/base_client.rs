use futures::stream::{BoxStream, StreamExt};
use reqwest::{Client, Response};
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;

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
    ) -> Result<Response, Box<dyn Error>> {
        let client = Client::builder().build()?;
        let url = format!("{}/{}", self.endpoint, path);

        let mut request = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");

        for (key, value) in &self.extra_headers {
            request = request.header(key, value);
        }

        let response = request.json(payload).send().await?;
        Ok(response)
    }

    pub async fn get_response_stream(
        &self,
        path: &str,
        payload: &impl Serialize,
    ) -> Result<BoxStream<'static, Result<String, Box<dyn Error + Send + Sync>>>, Box<dyn Error>>
    {
        let response = self.send_request(path, payload).await?;
        let stream = response.bytes_stream();

        let s = stream
            .map(|item| {
                item.map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
                    .and_then(|chunk| {
                        String::from_utf8(chunk.to_vec())
                            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
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
                                if let Ok(parsed) = serde_json::from_str::<StreamResponse>(data) {
                                    if let Some(choice) = parsed.choices.get(0) {
                                        if let Some(c) = &choice.delta.content {
                                            content.push_str(c);
                                        }
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
