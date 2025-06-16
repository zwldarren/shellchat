use crate::core::error::SchatError;
use futures::stream::{BoxStream, StreamExt};
use reqwest::{Client, Response};
use serde::Serialize;
use std::collections::HashMap;

/// Generic HTTP client that supports different authentication schemes
#[derive(Clone)]
pub struct HttpClient {
    base_url: String,
    auth_header: Option<(String, String)>,
    extra_headers: HashMap<String, String>,
    query_params: HashMap<String, String>,
}

impl HttpClient {
    pub fn new(
        base_url: String,
        auth_header: Option<(String, String)>,
        extra_headers: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            base_url,
            auth_header,
            extra_headers: extra_headers.unwrap_or_default(),
            query_params: HashMap::new(),
        }
    }

    /// Add a query parameter to the client
    pub fn add_query_param(&mut self, key: &str, value: String) {
        self.query_params.insert(key.to_string(), value);
    }

    /// Send a POST request with JSON payload
    pub async fn post<T: Serialize + ?Sized>(
        &self,
        path: &str,
        payload: &T,
    ) -> Result<Response, SchatError> {
        let client = Client::builder()
            .build()
            .map_err(|e| SchatError::Network(format!("Failed to create HTTP client: {}", e)))?;

        let mut url = format!("{}/{}", self.base_url, path);

        // Add query parameters if any
        if !self.query_params.is_empty() {
            url.push('?');
            let mut first = true;
            for (key, value) in &self.query_params {
                if !first {
                    url.push('&');
                }
                first = false;
                url.push_str(&format!("{}={}", key, value));
            }
        }

        let mut request = client.post(&url).header("Content-Type", "application/json");

        // Add authentication header if configured
        if let Some((key, value)) = &self.auth_header {
            request = request.header(key, value);
        }

        // Add extra headers
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

    /// Create a streaming response handler
    pub async fn stream_response<P>(
        &self,
        response: Response,
        parser: P,
    ) -> Result<BoxStream<'static, Result<String, SchatError>>, SchatError>
    where
        P: Fn(String) -> Result<Option<String>, SchatError> + Clone + Send + 'static,
    {
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
            .filter_map(move |res| {
                let parser = parser.clone();
                async move {
                    match res {
                        Ok(s) => match parser(s) {
                            Ok(Some(content)) => Some(Ok(content)),
                            Ok(None) => None,
                            Err(e) => Some(Err(e)),
                        },
                        Err(e) => Some(Err(e)),
                    }
                }
            });

        Ok(s.boxed())
    }
}
