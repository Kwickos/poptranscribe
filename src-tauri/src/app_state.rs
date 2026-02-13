use std::sync::{Arc, Mutex};
use crate::audio::capture::AudioCapturer;
use crate::db::Database;

/// Wrapper that asserts `Send + Sync` for `AudioCapturer`.
///
/// `cpal::Stream` (and `SCStream` on macOS) are marked `!Send` due to platform
/// constraints (`PhantomData<*mut ()>`), but we only ever access the capturer
/// through the `Mutex<Option<ActiveSession>>` in `AppState`, so cross-thread
/// access is properly synchronised.
pub struct SendCapturer(pub AudioCapturer);

// SAFETY: Access is serialised through `Mutex<Option<ActiveSession>>`.
// The capturer is created on the main thread and `stop()` is called while
// holding the same mutex, so there is no concurrent access.
unsafe impl Send for SendCapturer {}
unsafe impl Sync for SendCapturer {}

/// Tracks an active recording session, including the audio capturer and
/// accumulated raw samples.
pub struct ActiveSession {
    pub id: String,
    pub capturer: SendCapturer,
    pub audio_samples: Arc<Mutex<Vec<i16>>>,
    pub sample_rate: u32,
    pub stop_signal: tokio::sync::watch::Sender<bool>,
}

pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub api_key: Arc<Mutex<String>>,
    pub active_session: Mutex<Option<ActiveSession>>,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
            api_key: Arc::new(Mutex::new(String::new())),
            active_session: Mutex::new(None),
        }
    }
}
