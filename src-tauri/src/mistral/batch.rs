use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TranscriptionSegment {
    pub text: String,
    pub start: f64,
    pub end: f64,
    pub speaker: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TranscriptionResponse {
    pub text: String,
    pub segments: Vec<TranscriptionSegment>,
}
