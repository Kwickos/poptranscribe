use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite;

const REALTIME_MODEL: &str = "voxtral-mini-transcribe-realtime-2602";
const WS_BASE: &str = "wss://api.mistral.ai/v1/audio/transcriptions/realtime";

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
    /// Server-side error forwarded to the UI.
    #[serde(skip)]
    Error { message: String },
}

/// Internal enum to parse all WebSocket events including session & error.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum WsIncoming {
    #[serde(rename = "session.created")]
    SessionCreated {},
    #[serde(rename = "session.updated")]
    SessionUpdated {},
    #[serde(rename = "transcription.language")]
    Language { audio_language: String },
    #[serde(rename = "transcription.text.delta")]
    TextDelta { text: String },
    #[serde(rename = "transcription.segment")]
    Segment { text: String, start: f64, end: f64 },
    #[serde(rename = "transcription.done")]
    Done { text: String },
    #[serde(rename = "error")]
    Error { error: serde_json::Value },
}

/// Messages sent from the audio loop to the WebSocket sender task.
enum AudioMsg {
    Chunk(Vec<i16>),
    End,
}

/// Handle for sending audio to an active real-time transcription session.
pub struct RealtimeHandle {
    tx: mpsc::UnboundedSender<AudioMsg>,
}

impl RealtimeHandle {
    /// Send a chunk of i16 PCM samples to the transcription service.
    pub fn send_audio(&self, samples: Vec<i16>) {
        let _ = self.tx.send(AudioMsg::Chunk(samples));
    }

    /// Signal end of audio input.
    pub fn end_audio(&self) {
        let _ = self.tx.send(AudioMsg::End);
    }
}

/// Helper: build a tungstenite Text message from a string.
fn text_msg(s: String) -> tungstenite::Message {
    tungstenite::Message::Text(s.into())
}

/// Extract the text payload from a tungstenite Text message.
fn msg_text(msg: &tungstenite::Message) -> Option<String> {
    match msg {
        tungstenite::Message::Text(t) => Some(t.to_string()),
        _ => None,
    }
}

/// Resample i16 PCM from `from_rate` to `to_rate` using linear interpolation.
pub fn resample(samples: &[i16], from_rate: u32, to_rate: u32) -> Vec<i16> {
    if from_rate == to_rate || samples.is_empty() {
        return samples.to_vec();
    }
    let ratio = from_rate as f64 / to_rate as f64;
    let out_len = (samples.len() as f64 / ratio).ceil() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_pos = i as f64 * ratio;
        let idx = src_pos as usize;
        let frac = src_pos - idx as f64;
        let s = if idx + 1 < samples.len() {
            samples[idx] as f64 * (1.0 - frac) + samples[idx + 1] as f64 * frac
        } else {
            samples[idx.min(samples.len() - 1)] as f64
        };
        out.push(s.round() as i16);
    }
    out
}

/// Connect to Mistral real-time transcription WebSocket.
///
/// `source_sample_rate` is the rate of audio you will send via `send_audio()`.
/// Audio is resampled to 16kHz internally before being sent to the API.
///
/// Returns a `RealtimeHandle` for sending audio and a receiver for
/// transcription events. The WebSocket I/O runs in spawned tasks.
pub async fn connect_realtime(
    api_key: &str,
    source_sample_rate: u32,
) -> Result<
    (
        RealtimeHandle,
        mpsc::UnboundedReceiver<TranscriptionEvent>,
    ),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let url = format!("{}?model={}", WS_BASE, REALTIME_MODEL);

    let request = tungstenite::http::Request::builder()
        .uri(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Host", "api.mistral.ai")
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header(
            "Sec-WebSocket-Key",
            tungstenite::handshake::client::generate_key(),
        )
        .body(())?;

    let (ws_stream, _response) = tokio_tungstenite::connect_async(request)
        .await
        .map_err(|e| format!("WebSocket connection failed: {}", e))?;

    let (mut ws_write, mut ws_read) = ws_stream.split();

    // Wait for session.created
    let mut session_ready = false;
    while let Some(msg) = ws_read.next().await {
        let msg = msg.map_err(|e| format!("WebSocket read error: {}", e))?;
        if let Some(text) = msg_text(&msg) {
            eprintln!("[realtime] << {}", text);
            if let Ok(event) = serde_json::from_str::<WsIncoming>(&text) {
                match event {
                    WsIncoming::SessionCreated {} => {
                        session_ready = true;
                        break;
                    }
                    WsIncoming::Error { error } => {
                        return Err(
                            format!("Realtime session error: {}", error).into()
                        );
                    }
                    _ => {}
                }
            } else {
                eprintln!("[realtime] Unparseable message during handshake: {}", text);
            }
        }
    }
    if !session_ready {
        return Err("WebSocket closed before session.created".into());
    }

    eprintln!("[realtime] Session created, configuring audio format...");

    // Always send 16kHz to Mistral (API requirement)
    let api_sample_rate: u32 = 16000;
    let update = serde_json::json!({
        "type": "session.update",
        "session": {
            "audio_format": {
                "encoding": "pcm_s16le",
                "sample_rate": api_sample_rate
            }
        }
    });
    ws_write
        .send(text_msg(update.to_string()))
        .await
        .map_err(|e| format!("Failed to send session.update: {}", e))?;

    // Wait for session.updated acknowledgment
    let mut session_updated = false;
    while let Some(msg) = ws_read.next().await {
        let msg = msg.map_err(|e| format!("WebSocket read error: {}", e))?;
        if let Some(text) = msg_text(&msg) {
            eprintln!("[realtime] << {}", text);
            if let Ok(event) = serde_json::from_str::<WsIncoming>(&text) {
                match event {
                    WsIncoming::SessionUpdated {} => {
                        session_updated = true;
                        break;
                    }
                    WsIncoming::Error { error } => {
                        return Err(
                            format!("Realtime session.update error: {}", error).into()
                        );
                    }
                    _ => {}
                }
            } else {
                eprintln!("[realtime] Unparseable message after session.update: {}", text);
            }
        }
    }
    if !session_updated {
        return Err("WebSocket closed before session.updated".into());
    }

    eprintln!(
        "[realtime] Audio format set to pcm_s16le @ {} Hz (source: {} Hz)",
        api_sample_rate, source_sample_rate
    );

    // Channels
    let (audio_tx, mut audio_rx) = mpsc::unbounded_channel::<AudioMsg>();
    let (event_tx, event_rx) = mpsc::unbounded_channel::<TranscriptionEvent>();

    // Sender task: reads audio messages, resamples if needed, forwards to WebSocket
    let src_rate = source_sample_rate;
    tokio::spawn(async move {
        let b64 = base64::engine::general_purpose::STANDARD;
        while let Some(msg) = audio_rx.recv().await {
            match msg {
                AudioMsg::Chunk(samples) => {
                    // Resample to 16kHz if source rate differs
                    let resampled = resample(&samples, src_rate, 16000);

                    // Convert i16 samples to little-endian bytes
                    let mut bytes = Vec::with_capacity(resampled.len() * 2);
                    for &s in &resampled {
                        bytes.extend_from_slice(&s.to_le_bytes());
                    }
                    let encoded = b64.encode(&bytes);
                    let json = serde_json::json!({
                        "type": "input_audio.append",
                        "audio": encoded
                    });
                    if ws_write
                        .send(text_msg(json.to_string()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                AudioMsg::End => {
                    let json = serde_json::json!({"type": "input_audio.end"});
                    let _ = ws_write
                        .send(text_msg(json.to_string()))
                        .await;
                    break;
                }
            }
        }
    });

    // Receiver task: reads WebSocket events and forwards to event channel
    tokio::spawn(async move {
        while let Some(msg) = ws_read.next().await {
            let msg = match msg {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("[realtime] WebSocket read error: {}", e);
                    let _ = event_tx.send(TranscriptionEvent::Error {
                        message: format!("WebSocket read error: {}", e),
                    });
                    break;
                }
            };
            if let Some(text) = msg_text(&msg) {
                eprintln!("[realtime] << {}", text);
                match serde_json::from_str::<WsIncoming>(&text) {
                    Ok(ws_event) => match ws_event {
                        WsIncoming::TextDelta { text } => {
                            let _ =
                                event_tx.send(TranscriptionEvent::TextDelta { text });
                        }
                        WsIncoming::Segment { text, start, end } => {
                            let _ = event_tx.send(
                                TranscriptionEvent::Segment { text, start, end },
                            );
                        }
                        WsIncoming::Done { text } => {
                            let _ =
                                event_tx.send(TranscriptionEvent::Done { text });
                            break;
                        }
                        WsIncoming::Language { audio_language } => {
                            let _ = event_tx
                                .send(TranscriptionEvent::Language { audio_language });
                        }
                        WsIncoming::Error { error } => {
                            eprintln!("[realtime] Error from server: {}", error);
                            let _ = event_tx.send(TranscriptionEvent::Error {
                                message: format!("Erreur serveur: {}", error),
                            });
                            break;
                        }
                        _ => {} // session.updated
                    },
                    Err(e) => {
                        eprintln!("[realtime] Failed to parse message: {} — raw: {}", e, text);
                    }
                }
            } else if matches!(msg, tungstenite::Message::Close(_)) {
                eprintln!("[realtime] WebSocket closed by server");
                break;
            }
        }
    });

    Ok((RealtimeHandle { tx: audio_tx }, event_rx))
}

/// Stream transcription events from Mistral HTTP API (file upload).
/// Used for batch re-processing, NOT for real-time.
pub async fn transcribe_stream(
    api_key: &str,
    audio_path: &std::path::Path,
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

    let form = reqwest::multipart::Form::new()
        .text("model", "voxtral-mini-latest")
        .text("stream", "true")
        .text("timestamp_granularities", "segment")
        .part("file", file_part);

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

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = buffer.find("\n\n") {
            let event_str = buffer[..pos].to_string();
            buffer = buffer[pos + 2..].to_string();

            for line in event_str.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data.trim() == "[DONE]" {
                        return Ok(());
                    }
                    if let Ok(event) = serde_json::from_str::<TranscriptionEvent>(data)
                    {
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
        let data =
            r#"{"type":"transcription.segment","text":"Bonjour","start":0.0,"end":1.5}"#;
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

    #[test]
    fn test_parse_ws_session_created() {
        let data = r#"{"type":"session.created","session":{}}"#;
        let event: WsIncoming = serde_json::from_str(data).unwrap();
        assert!(matches!(event, WsIncoming::SessionCreated { .. }));
    }

    #[test]
    fn test_parse_ws_error() {
        let data =
            r#"{"type":"error","error":{"message":"Invalid request","code":"invalid"}}"#;
        let event: WsIncoming = serde_json::from_str(data).unwrap();
        assert!(matches!(event, WsIncoming::Error { .. }));
    }
}
