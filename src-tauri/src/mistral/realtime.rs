use serde::Deserialize;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum TranscriptionEvent {
    #[serde(rename = "transcription.language")]
    Language { audio_language: String },
    #[serde(rename = "transcription.text.delta")]
    TextDelta { text: String },
    #[serde(rename = "transcription.segment")]
    Segment { text: String, start: f64, end: f64 },
    #[serde(rename = "transcription.done")]
    Done { text: String },
}

/// Stream transcription events from Mistral API.
/// Sends TranscriptionEvent through the provided channel as they arrive.
pub async fn transcribe_stream(
    api_key: &str,
    audio_path: &std::path::Path,
    language: Option<&str>,
    tx: mpsc::UnboundedSender<TranscriptionEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let file_bytes = tokio::fs::read(audio_path).await?;
    let file_name = audio_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("audio.wav")
        .to_string();

    let file_part = reqwest::multipart::Part::bytes(file_bytes)
        .file_name(file_name)
        .mime_str("audio/wav")?;

    let mut form = reqwest::multipart::Form::new()
        .text("model", "voxtral-mini-latest")
        .text("stream", "true")
        .text("timestamp_granularities", "segment")
        .part("file", file_part);

    // Note: language param is incompatible with timestamp_granularities per Mistral docs.
    // The API auto-detects language, so we omit it.
    let _ = language; // keep parameter for API compat

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

    // Parse SSE events from response body
    // SSE format: "data: {json}\n\n" or "data: [DONE]\n\n"
    use futures_util::StreamExt;
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        // Process complete SSE events (separated by double newlines)
        while let Some(pos) = buffer.find("\n\n") {
            let event_str = buffer[..pos].to_string();
            buffer = buffer[pos + 2..].to_string();

            for line in event_str.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data.trim() == "[DONE]" {
                        return Ok(());
                    }
                    if let Ok(event) = serde_json::from_str::<TranscriptionEvent>(data) {
                        let _ = tx.send(event);
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_language_event() {
        let data = r#"{"type":"transcription.language","audio_language":"fr"}"#;
        let event: TranscriptionEvent = serde_json::from_str(data).unwrap();
        match event {
            TranscriptionEvent::Language { audio_language } => {
                assert_eq!(audio_language, "fr");
            }
            _ => panic!("Expected Language event"),
        }
    }

    #[test]
    fn test_parse_segment_event() {
        let data = r#"{"type":"transcription.segment","text":"Bonjour","start":0.0,"end":1.5}"#;
        let event: TranscriptionEvent = serde_json::from_str(data).unwrap();
        match event {
            TranscriptionEvent::Segment { text, start, end } => {
                assert_eq!(text, "Bonjour");
                assert_eq!(start, 0.0);
                assert_eq!(end, 1.5);
            }
            _ => panic!("Expected Segment event"),
        }
    }

    #[test]
    fn test_parse_text_delta_event() {
        let data = r#"{"type":"transcription.text.delta","text":"Bonjour "}"#;
        let event: TranscriptionEvent = serde_json::from_str(data).unwrap();
        match event {
            TranscriptionEvent::TextDelta { text } => {
                assert_eq!(text, "Bonjour ");
            }
            _ => panic!("Expected TextDelta event"),
        }
    }

    #[test]
    fn test_parse_done_event() {
        let data = r#"{"type":"transcription.done","text":"Bonjour tout le monde"}"#;
        let event: TranscriptionEvent = serde_json::from_str(data).unwrap();
        match event {
            TranscriptionEvent::Done { text } => {
                assert_eq!(text, "Bonjour tout le monde");
            }
            _ => panic!("Expected Done event"),
        }
    }
}
