use reqwest::multipart;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TranscriptionSegment {
    pub text: String,
    pub start: f64,
    pub end: f64,
    #[serde(alias = "speaker")]
    pub speaker_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TranscriptionResponse {
    pub text: String,
    #[serde(default)]
    pub segments: Vec<TranscriptionSegment>,
}

pub async fn transcribe_batch(
    api_key: &str,
    audio_path: &std::path::Path,
    diarize: bool,
    language: Option<&str>,
) -> Result<TranscriptionResponse, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let file_bytes = tokio::fs::read(audio_path).await?;
    let file_name = audio_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("audio.wav")
        .to_string();

    let file_part = multipart::Part::bytes(file_bytes)
        .file_name(file_name)
        .mime_str("audio/wav")?;

    let mut form = multipart::Form::new()
        .text("model", "voxtral-mini-latest")
        .part("file", file_part)
        .text("timestamp_granularities", "segment");

    if diarize {
        form = form.text("diarize", "true");
    }
    // Note: language param is incompatible with timestamp_granularities per Mistral docs.
    // The API auto-detects language, so we omit it.

    let response = client
        .post("https://api.mistral.ai/v1/audio/transcriptions")
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Mistral API error {}: {}", status, body).into());
    }

    let result = response.json::<TranscriptionResponse>().await?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_transcription_response_with_speaker_id() {
        // Mistral API returns speaker_id (not speaker)
        let json = r#"{
            "text": "Bonjour tout le monde. Comment allez-vous ?",
            "segments": [
                {
                    "text": "Bonjour tout le monde.",
                    "start": 0.0,
                    "end": 2.5,
                    "speaker_id": "speaker_1"
                },
                {
                    "text": "Comment allez-vous ?",
                    "start": 2.5,
                    "end": 4.0,
                    "speaker_id": "speaker_2"
                }
            ]
        }"#;

        let response: TranscriptionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.segments.len(), 2);
        assert_eq!(response.segments[0].speaker_id.as_deref(), Some("speaker_1"));
        assert_eq!(response.segments[1].text, "Comment allez-vous ?");
    }

    #[test]
    fn test_deserialize_transcription_response_with_speaker_alias() {
        // Also accepts "speaker" as alias for backwards compat
        let json = r#"{
            "text": "Hello",
            "segments": [
                {
                    "text": "Hello",
                    "start": 0.0,
                    "end": 1.0,
                    "speaker": "Speaker 1"
                }
            ]
        }"#;

        let response: TranscriptionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.segments[0].speaker_id.as_deref(), Some("Speaker 1"));
    }

    #[test]
    fn test_deserialize_response_without_segments() {
        let json = r#"{"text": "Hello world"}"#;
        let response: TranscriptionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.text, "Hello world");
        assert!(response.segments.is_empty());
    }
}
