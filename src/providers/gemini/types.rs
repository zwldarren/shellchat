use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiContent {
    pub parts: Vec<GeminiPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiPart {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct GeminiContentPart {
    pub role: String,
    pub parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
pub struct GeminiRequest {
    pub contents: Vec<GeminiContentPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<SystemInstruction>,
}

#[derive(Debug, Serialize)]
pub struct SystemInstruction {
    pub parts: Vec<GeminiPart>,
}

#[derive(Debug, Deserialize)]
pub struct GeminiResponse {
    pub candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
pub struct GeminiCandidate {
    pub content: GeminiContent,
}
