use serde::Deserialize;

#[derive(Debug, Deserialize)]
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
