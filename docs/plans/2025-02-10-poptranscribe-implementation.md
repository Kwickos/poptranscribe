# PopTranscribe Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a macOS menu bar app that transcribes meetings in real-time with search, then reprocesses with speaker diarization post-meeting.

**Architecture:** Tauri 2 (Rust backend + React/TypeScript frontend). Rust handles audio capture (ScreenCaptureKit + cpal), Mistral API calls (direct HTTP via reqwest), and SQLite storage. React handles UI with three views: live session, history, and detail.

**Tech Stack:** Tauri 2, Rust, React/TypeScript, Vite, Tailwind CSS, screencapturekit-rs, cpal, reqwest, rusqlite, Mistral API (Voxtral Mini)

**Design doc:** `docs/plans/2025-02-10-poptranscribe-design.md`

---

## Phase 1: Project Scaffolding

### Task 1: Create Tauri 2 + React project

**Files:**
- Create: entire project structure via CLI

**Step 1: Scaffold the project**

Run:
```bash
npm create tauri-app@latest poptranscribe-app -- --template react-ts
```
Select: TypeScript, npm, React, TypeScript

**Step 2: Move contents to project root**

Move the generated files from `poptranscribe-app/` into the project root `/Users/Alexandre/Projects/poptranscribe/`, preserving the existing `docs/` folder.

**Step 3: Install frontend dependencies**

Run:
```bash
cd /Users/Alexandre/Projects/poptranscribe
npm install
```

**Step 4: Add Tailwind CSS**

Run:
```bash
npm install -D tailwindcss @tailwindcss/vite
```

Configure Tailwind in `vite.config.ts` and add `@import "tailwindcss"` to `src/styles.css`.

**Step 5: Add Rust dependencies**

Edit `src-tauri/Cargo.toml` to add:
```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "multipart", "stream"] }
rusqlite = { version = "0.31", features = ["bundled"] }
screencapturekit = { version = "1", features = ["async"] }
cpal = "0.15"
hound = "3.5"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

**Step 6: Verify it builds**

Run:
```bash
cd /Users/Alexandre/Projects/poptranscribe
npm run tauri dev
```
Expected: Tauri window opens with default React page.

**Step 7: Init git and commit**

```bash
git init
git add -A
git commit -m "feat: scaffold Tauri 2 + React/TS project with dependencies"
```

---

### Task 2: Set up Rust module structure

**Files:**
- Create: `src-tauri/src/audio/mod.rs`
- Create: `src-tauri/src/audio/capture.rs`
- Create: `src-tauri/src/audio/mixer.rs`
- Create: `src-tauri/src/audio/store.rs`
- Create: `src-tauri/src/mistral/mod.rs`
- Create: `src-tauri/src/mistral/realtime.rs`
- Create: `src-tauri/src/mistral/batch.rs`
- Create: `src-tauri/src/mistral/chat.rs`
- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/export/mod.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Create module files with stub implementations**

Create the directory structure and module files. Each module gets a `mod.rs` with public re-exports and stub functions that return `todo!()`.

`src-tauri/src/audio/mod.rs`:
```rust
pub mod capture;
pub mod mixer;
pub mod store;
```

`src-tauri/src/audio/capture.rs`:
```rust
pub enum CaptureMode {
    Visio,       // ScreenCaptureKit + mic
    InPerson,    // mic only
}

pub struct AudioCapturer {
    mode: CaptureMode,
}

impl AudioCapturer {
    pub fn new(mode: CaptureMode) -> Self {
        Self { mode }
    }
}
```

Similar stubs for all other modules.

**Step 2: Update main.rs to declare modules**

```rust
mod audio;
mod mistral;
mod db;
mod export;
```

**Step 3: Verify it compiles**

Run: `cd /Users/Alexandre/Projects/poptranscribe && cargo build --manifest-path src-tauri/Cargo.toml`
Expected: Compiles with no errors (warnings OK for unused code).

**Step 4: Commit**

```bash
git add src-tauri/src/
git commit -m "feat: set up Rust module structure with stubs"
```

---

## Phase 2: Audio Capture

### Task 3: Implement microphone capture with cpal

**Files:**
- Modify: `src-tauri/src/audio/capture.rs`
- Create: `src-tauri/tests/audio_capture_test.rs` (integration test)

**Step 1: Write integration test for mic capture**

```rust
// src-tauri/tests/audio_capture_test.rs
#[test]
fn test_mic_capture_produces_samples() {
    use poptranscribe::audio::capture::{AudioCapturer, CaptureMode};
    let capturer = AudioCapturer::new(CaptureMode::InPerson);
    let receiver = capturer.start().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(1));
    capturer.stop();
    let samples: Vec<i16> = receiver.try_iter().collect();
    assert!(!samples.is_empty(), "Should have captured audio samples");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml test_mic_capture`
Expected: FAIL - methods not implemented.

**Step 3: Implement mic capture using cpal**

In `capture.rs`, implement:
- `start()` -> opens default input device via cpal
- Configures stream: PCM i16, 16kHz mono
- Sends samples via `std::sync::mpsc::Sender<Vec<i16>>`
- `stop()` -> stops the stream

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml test_mic_capture`
Expected: PASS (requires microphone permission on macOS).

**Step 5: Commit**

```bash
git add src-tauri/src/audio/capture.rs src-tauri/tests/
git commit -m "feat: implement microphone capture via cpal"
```

---

### Task 4: Implement system audio capture with ScreenCaptureKit

**Files:**
- Modify: `src-tauri/src/audio/capture.rs`

**Step 1: Write integration test for system audio capture**

```rust
#[test]
fn test_system_audio_capture_initializes() {
    use poptranscribe::audio::capture::{AudioCapturer, CaptureMode};
    let capturer = AudioCapturer::new(CaptureMode::Visio);
    // In Visio mode, should initialize both system audio and mic
    assert!(capturer.start().is_ok());
    capturer.stop();
}
```

**Step 2: Run test to verify it fails**

Expected: FAIL - Visio mode not implemented.

**Step 3: Implement ScreenCaptureKit capture**

Using `screencapturekit` crate:
- Create `SCStream` with `with_captures_audio(true)`, sample rate 16kHz
- Set up audio handler that receives `CMSampleBuffer`
- Extract PCM i16 samples from the audio buffer
- Send via same channel as mic capture

**Step 4: Run test to verify it passes**

Expected: PASS (requires Screen Recording permission).

**Step 5: Commit**

```bash
git add src-tauri/src/audio/capture.rs src-tauri/tests/
git commit -m "feat: implement system audio capture via ScreenCaptureKit"
```

---

### Task 5: Implement audio mixer

**Files:**
- Modify: `src-tauri/src/audio/mixer.rs`

**Step 1: Write unit test for mixing**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mix_two_streams() {
        let a: Vec<i16> = vec![1000, 2000, 3000];
        let b: Vec<i16> = vec![500, 1000, 1500];
        let mixed = mix_samples(&a, &b);
        assert_eq!(mixed, vec![1500, 3000, 4500]);
    }

    #[test]
    fn test_mix_clamps_to_i16_max() {
        let a: Vec<i16> = vec![i16::MAX];
        let b: Vec<i16> = vec![1000];
        let mixed = mix_samples(&a, &b);
        assert_eq!(mixed, vec![i16::MAX]);
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml test_mix`
Expected: FAIL.

**Step 3: Implement mix_samples**

```rust
pub fn mix_samples(a: &[i16], b: &[i16]) -> Vec<i16> {
    let len = a.len().max(b.len());
    (0..len)
        .map(|i| {
            let sa = *a.get(i).unwrap_or(&0) as i32;
            let sb = *b.get(i).unwrap_or(&0) as i32;
            (sa + sb).clamp(i16::MIN as i32, i16::MAX as i32) as i16
        })
        .collect()
}
```

**Step 4: Run tests to verify they pass**

Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/src/audio/mixer.rs
git commit -m "feat: implement audio mixer with clamping"
```

---

### Task 6: Implement WAV file storage

**Files:**
- Modify: `src-tauri/src/audio/store.rs`

**Step 1: Write test**

```rust
#[test]
fn test_save_and_read_wav() {
    let samples: Vec<i16> = (0..16000).map(|i| (i % 1000) as i16).collect();
    let path = std::env::temp_dir().join("test_audio.wav");
    save_wav(&path, &samples, 16000).unwrap();
    assert!(path.exists());
    let metadata = std::fs::metadata(&path).unwrap();
    assert!(metadata.len() > 0);
    std::fs::remove_file(&path).ok();
}
```

**Step 2: Run test, verify fail**

**Step 3: Implement using hound crate**

```rust
use hound::{WavWriter, WavSpec, SampleFormat};

pub fn save_wav(path: &std::path::Path, samples: &[i16], sample_rate: u32) -> Result<(), hound::Error> {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(path, spec)?;
    for &sample in samples {
        writer.write_sample(sample)?;
    }
    writer.finalize()?;
    Ok(())
}
```

**Step 4: Run test, verify pass**

**Step 5: Commit**

```bash
git add src-tauri/src/audio/store.rs
git commit -m "feat: implement WAV file storage with hound"
```

---

## Phase 3: Mistral API Client

### Task 7: Implement batch transcription client

**Files:**
- Modify: `src-tauri/src/mistral/batch.rs`

**Step 1: Write test (mocked)**

```rust
#[tokio::test]
async fn test_batch_transcribe_request_format() {
    // Test that the multipart form is correctly constructed
    let request = build_transcription_request(
        "/tmp/test.wav",
        "voxtral-mini-latest",
        true, // diarize
        Some("fr"),
    );
    assert!(request.is_ok());
}
```

**Step 2: Run test, verify fail**

**Step 3: Implement batch client**

```rust
use reqwest::multipart;
use serde::{Deserialize, Serialize};

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

pub async fn transcribe_batch(
    api_key: &str,
    audio_path: &std::path::Path,
    diarize: bool,
    language: Option<&str>,
) -> Result<TranscriptionResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let file_bytes = tokio::fs::read(audio_path).await.unwrap();
    let file_part = multipart::Part::bytes(file_bytes)
        .file_name("audio.wav")
        .mime_str("audio/wav").unwrap();

    let mut form = multipart::Form::new()
        .text("model", "voxtral-mini-latest")
        .part("file", file_part);

    if diarize {
        form = form.text("diarize", "true");
    }
    if let Some(lang) = language {
        form = form.text("language", lang.to_string());
    }
    form = form.text("timestamp_granularities", "[\"segment\",\"word\"]");

    let response = client
        .post("https://api.mistral.ai/v1/audio/transcriptions")
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .await?
        .json::<TranscriptionResponse>()
        .await?;

    Ok(response)
}
```

**Step 4: Run test, verify pass**

**Step 5: Commit**

```bash
git add src-tauri/src/mistral/batch.rs
git commit -m "feat: implement Mistral batch transcription client"
```

---

### Task 8: Implement SSE streaming transcription client

**Files:**
- Modify: `src-tauri/src/mistral/realtime.rs`
- Add dep: `reqwest-eventsource` or manual SSE parsing

**Step 1: Add dependency**

Add to `Cargo.toml`:
```toml
reqwest-eventsource = "0.6"
futures-util = "0.3"
```

**Step 2: Write test for SSE event parsing**

```rust
#[test]
fn test_parse_transcription_event() {
    let data = r#"{"type":"transcription.segment","text":"Bonjour tout le monde","start":0.0,"end":2.5}"#;
    let event: TranscriptionEvent = serde_json::from_str(data).unwrap();
    match event {
        TranscriptionEvent::Segment { text, start, end } => {
            assert_eq!(text, "Bonjour tout le monde");
            assert_eq!(start, 0.0);
            assert_eq!(end, 2.5);
        }
        _ => panic!("Expected Segment event"),
    }
}
```

**Step 3: Run test, verify fail**

**Step 4: Implement SSE client**

```rust
use serde::Deserialize;
use tokio::sync::mpsc;

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

pub async fn transcribe_stream(
    api_key: &str,
    audio_path: &std::path::Path,
    tx: mpsc::UnboundedSender<TranscriptionEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let file_bytes = tokio::fs::read(audio_path).await?;
    let file_part = reqwest::multipart::Part::bytes(file_bytes)
        .file_name("audio.wav")
        .mime_str("audio/wav")?;

    let form = reqwest::multipart::Form::new()
        .text("model", "voxtral-mini-latest")
        .text("stream", "true")
        .part("file", file_part);

    let response = client
        .post("https://api.mistral.ai/v1/audio/transcriptions")
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .await?;

    // Parse SSE events from response body
    let mut stream = response.bytes_stream();
    // ... parse SSE lines, deserialize to TranscriptionEvent, send via tx
    Ok(())
}
```

**Step 5: Run test, verify pass**

**Step 6: Commit**

```bash
git add src-tauri/src/mistral/realtime.rs src-tauri/Cargo.toml
git commit -m "feat: implement SSE streaming transcription client"
```

---

### Task 9: Implement Mistral chat client (search + summary)

**Files:**
- Modify: `src-tauri/src/mistral/chat.rs`

**Step 1: Write test for chat request**

```rust
#[test]
fn test_build_search_prompt() {
    let transcript = "Speaker 1: On devrait revoir le budget.\nSpeaker 2: Oui le budget Q3 est trop eleve.";
    let query = "Qu'est-ce qui a ete dit sur le budget ?";
    let prompt = build_search_prompt(transcript, query);
    assert!(prompt.contains(transcript));
    assert!(prompt.contains(query));
}
```

**Step 2: Run test, verify fail**

**Step 3: Implement chat client**

Two functions:
- `search_transcript(api_key, transcript, query) -> String` : envoie la transcription + question a Mistral, retourne la reponse
- `generate_summary(api_key, transcript) -> Summary` : genere points cles, decisions, actions

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Summary {
    pub key_points: Vec<String>,
    pub decisions: Vec<String>,
    pub action_items: Vec<ActionItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionItem {
    pub description: String,
    pub assignee: Option<String>,
}
```

**Step 4: Run test, verify pass**

**Step 5: Commit**

```bash
git add src-tauri/src/mistral/chat.rs
git commit -m "feat: implement Mistral chat client for search and summary"
```

---

## Phase 4: Database

### Task 10: Implement SQLite schema and CRUD

**Files:**
- Modify: `src-tauri/src/db/mod.rs`

**Step 1: Write tests**

```rust
#[test]
fn test_create_and_get_session() {
    let db = Database::new_in_memory().unwrap();
    let session_id = db.create_session("Test Meeting", "visio").unwrap();
    let session = db.get_session(&session_id).unwrap();
    assert_eq!(session.title, "Test Meeting");
    assert_eq!(session.mode, "visio");
}

#[test]
fn test_save_and_get_transcription() {
    let db = Database::new_in_memory().unwrap();
    let session_id = db.create_session("Test", "visio").unwrap();
    db.save_segment(&session_id, "Bonjour", 0.0, 1.5, None).unwrap();
    let segments = db.get_segments(&session_id).unwrap();
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].text, "Bonjour");
}
```

**Step 2: Run tests, verify fail**

**Step 3: Implement Database struct**

Schema:
```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    mode TEXT NOT NULL,
    audio_path TEXT,
    created_at TEXT NOT NULL,
    duration_secs REAL,
    summary_json TEXT
);

CREATE TABLE segments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id),
    text TEXT NOT NULL,
    start_time REAL NOT NULL,
    end_time REAL NOT NULL,
    speaker TEXT,
    is_diarized INTEGER NOT NULL DEFAULT 0
);

CREATE VIRTUAL TABLE segments_fts USING fts5(text, content=segments, content_rowid=id);
```

Implement: `Database::new(path)`, `new_in_memory()`, `create_session()`, `get_session()`, `list_sessions()`, `save_segment()`, `get_segments()`, `save_summary()`, `search_text()`.

**Step 4: Run tests, verify pass**

**Step 5: Commit**

```bash
git add src-tauri/src/db/
git commit -m "feat: implement SQLite database with FTS5 search"
```

---

## Phase 5: Tauri Commands (Rust <-> React bridge)

### Task 11: Expose Tauri commands

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Define Tauri commands**

```rust
use tauri::State;

#[tauri::command]
async fn start_session(mode: String, state: State<'_, AppState>) -> Result<String, String> {
    // Create session in DB, start audio capture, start streaming transcription
    todo!()
}

#[tauri::command]
async fn stop_session(session_id: String, state: State<'_, AppState>) -> Result<(), String> {
    // Stop capture, trigger batch reprocessing
    todo!()
}

#[tauri::command]
async fn get_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, String> {
    todo!()
}

#[tauri::command]
async fn get_session_detail(session_id: String, state: State<'_, AppState>) -> Result<SessionDetail, String> {
    todo!()
}

#[tauri::command]
async fn search_text(query: String, session_id: Option<String>, state: State<'_, AppState>) -> Result<Vec<SearchResult>, String> {
    todo!()
}

#[tauri::command]
async fn search_llm(query: String, session_id: String, state: State<'_, AppState>) -> Result<String, String> {
    todo!()
}

#[tauri::command]
async fn rename_speaker(session_id: String, old_name: String, new_name: String, state: State<'_, AppState>) -> Result<(), String> {
    todo!()
}

#[tauri::command]
async fn export_session(session_id: String, format: String, state: State<'_, AppState>) -> Result<String, String> {
    todo!()
}

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    todo!()
}

#[tauri::command]
async fn save_settings(settings: Settings, state: State<'_, AppState>) -> Result<(), String> {
    todo!()
}
```

**Step 2: Register commands in main.rs**

```rust
fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            start_session,
            stop_session,
            get_sessions,
            get_session_detail,
            search_text,
            search_llm,
            rename_speaker,
            export_session,
            get_settings,
            save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 3: Verify it compiles**

Run: `cargo build --manifest-path src-tauri/Cargo.toml`

**Step 4: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "feat: define Tauri command interface"
```

---

## Phase 6: React Frontend

### Task 12: Set up React app structure + routing

**Files:**
- Modify: `src/App.tsx`
- Create: `src/views/SessionView.tsx`
- Create: `src/views/HistoryView.tsx`
- Create: `src/views/DetailView.tsx`
- Create: `src/views/SettingsView.tsx`
- Create: `src/components/Layout.tsx`
- Create: `src/hooks/useTauri.ts`
- Create: `src/types.ts`

**Step 1: Install React Router**

Run: `npm install react-router-dom`

**Step 2: Create types.ts with shared types**

```typescript
export interface Session {
  id: string;
  title: string;
  mode: 'visio' | 'inperson';
  created_at: string;
  duration_secs: number | null;
}

export interface Segment {
  id: number;
  text: string;
  start_time: number;
  end_time: number;
  speaker: string | null;
}

export interface SessionDetail extends Session {
  segments: Segment[];
  summary: Summary | null;
}

export interface Summary {
  key_points: string[];
  decisions: string[];
  action_items: ActionItem[];
}

export interface ActionItem {
  description: string;
  assignee: string | null;
}
```

**Step 3: Create Layout with sidebar navigation**

Simple layout: sidebar with nav links (Session, History, Settings) + main content area.

**Step 4: Create stub views**

Each view returns a placeholder div with the view name.

**Step 5: Wire up App.tsx with router**

```tsx
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import Layout from './components/Layout';
import SessionView from './views/SessionView';
import HistoryView from './views/HistoryView';
import DetailView from './views/DetailView';
import SettingsView from './views/SettingsView';

function App() {
  return (
    <BrowserRouter>
      <Layout>
        <Routes>
          <Route path="/" element={<SessionView />} />
          <Route path="/history" element={<HistoryView />} />
          <Route path="/session/:id" element={<DetailView />} />
          <Route path="/settings" element={<SettingsView />} />
        </Routes>
      </Layout>
    </BrowserRouter>
  );
}
```

**Step 6: Create useTauri.ts hook**

Wrapper around `@tauri-apps/api/core` `invoke()` for type-safe Tauri command calls.

**Step 7: Verify app runs**

Run: `npm run tauri dev`
Expected: App opens with layout and navigation.

**Step 8: Commit**

```bash
git add src/ package.json package-lock.json
git commit -m "feat: set up React app structure with routing and views"
```

---

### Task 13: Implement SessionView (live transcription)

**Files:**
- Modify: `src/views/SessionView.tsx`
- Create: `src/components/TranscriptLine.tsx`
- Create: `src/components/SearchBar.tsx`
- Create: `src/components/AudioLevel.tsx`

**Step 1: Build the SessionView UI**

Components:
- Mode toggle (Visio / In-Person) at the top
- Big Start/Stop button
- Timer showing elapsed time
- Audio level indicator (small VU meter)
- Scrolling transcript area with `TranscriptLine` components
- Search bar at top of transcript area
- Side panel for LLM search results

**Step 2: Wire up Tauri events for live transcription**

Use `@tauri-apps/api/event` `listen()` to receive real-time transcript segments from Rust backend:

```typescript
import { listen } from '@tauri-apps/api/event';

useEffect(() => {
  const unlisten = listen<Segment>('transcription-segment', (event) => {
    setSegments(prev => [...prev, event.payload]);
  });
  return () => { unlisten.then(fn => fn()); };
}, []);
```

**Step 3: Wire up Start/Stop to Tauri commands**

```typescript
const handleStart = async () => {
  const sessionId = await invoke<string>('start_session', { mode });
  setSessionId(sessionId);
  setIsRecording(true);
};

const handleStop = async () => {
  await invoke('stop_session', { sessionId });
  setIsRecording(false);
};
```

**Step 4: Implement search (text + LLM)**

- Text search: filter segments client-side with highlight
- LLM search: call `search_llm` Tauri command, display result in side panel

**Step 5: Verify it renders**

Run: `npm run tauri dev`
Expected: SessionView renders with all components (non-functional without backend wiring).

**Step 6: Commit**

```bash
git add src/
git commit -m "feat: implement SessionView with live transcription UI"
```

---

### Task 14: Implement HistoryView

**Files:**
- Modify: `src/views/HistoryView.tsx`
- Create: `src/components/SessionCard.tsx`

**Step 1: Build HistoryView**

- List of `SessionCard` components showing: title, date, duration, mode icon
- Search bar for global search across all sessions
- Click on card navigates to `/session/:id`

**Step 2: Wire up to Tauri**

```typescript
const [sessions, setSessions] = useState<Session[]>([]);
useEffect(() => {
  invoke<Session[]>('get_sessions').then(setSessions);
}, []);
```

**Step 3: Commit**

```bash
git add src/
git commit -m "feat: implement HistoryView with session list"
```

---

### Task 15: Implement DetailView (post-meeting)

**Files:**
- Modify: `src/views/DetailView.tsx`
- Create: `src/components/DiarizedTranscript.tsx`
- Create: `src/components/SummaryPanel.tsx`
- Create: `src/components/ExportButtons.tsx`
- Create: `src/components/SpeakerEditor.tsx`

**Step 1: Build DetailView**

- Header: title (editable), date, duration
- DiarizedTranscript: segments with speaker labels, timestamps, color-coded by speaker
- SpeakerEditor: click on speaker name to rename
- SummaryPanel: key points, decisions, action items
- ExportButtons: Markdown, PDF, Notion, Slack

**Step 2: Wire up to Tauri**

```typescript
const { id } = useParams();
const [detail, setDetail] = useState<SessionDetail | null>(null);
useEffect(() => {
  invoke<SessionDetail>('get_session_detail', { sessionId: id }).then(setDetail);
}, [id]);
```

**Step 3: Implement export buttons**

Each button calls `export_session` with the appropriate format.

**Step 4: Commit**

```bash
git add src/
git commit -m "feat: implement DetailView with diarized transcript and exports"
```

---

## Phase 7: Wiring It All Together

### Task 16: Implement start_session command (full pipeline)

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Create: `src-tauri/src/app_state.rs`

**Step 1: Create AppState**

```rust
pub struct AppState {
    pub db: Mutex<Database>,
    pub active_session: Mutex<Option<ActiveSession>>,
    pub api_key: Mutex<String>,
}

pub struct ActiveSession {
    pub id: String,
    pub capturer: AudioCapturer,
    pub audio_buffer: Arc<Mutex<Vec<i16>>>,
}
```

**Step 2: Implement start_session**

1. Create session in DB
2. Start audio capture (based on mode)
3. Spawn task: accumulate audio chunks, periodically send to Mistral streaming API
4. Emit `transcription-segment` events to frontend via `app.emit()`
5. Save audio to buffer for WAV export later

**Step 3: Implement stop_session**

1. Stop audio capture
2. Save WAV file from buffer
3. Spawn background task: batch transcription with diarization
4. On batch complete: save diarized segments to DB, generate summary
5. Emit `session-complete` event to frontend

**Step 4: Verify full flow works**

Run: `npm run tauri dev`
Test: Start a session, speak, stop, check that transcription appears.

**Step 5: Commit**

```bash
git add src-tauri/src/
git commit -m "feat: wire up full session pipeline (capture -> transcribe -> diarize)"
```

---

### Task 17: Implement search commands

**Files:**
- Modify: `src-tauri/src/commands.rs`

**Step 1: Implement search_text**

Uses SQLite FTS5 for fast text search. Returns matching segments with highlights.

**Step 2: Implement search_llm**

Fetches all segments for the session, builds prompt, calls Mistral chat API.

**Step 3: Test both search modes**

**Step 4: Commit**

```bash
git add src-tauri/src/commands.rs
git commit -m "feat: implement text and LLM search commands"
```

---

## Phase 8: Export

### Task 18: Implement export module

**Files:**
- Modify: `src-tauri/src/export/mod.rs`

**Step 1: Write tests**

```rust
#[test]
fn test_export_markdown() {
    let segments = vec![
        Segment { text: "Bonjour".into(), speaker: Some("Speaker 1".into()), start: 0.0, end: 1.0 },
    ];
    let summary = Summary { key_points: vec!["Test".into()], decisions: vec![], action_items: vec![] };
    let md = export_markdown("Reunion Test", &segments, &summary);
    assert!(md.contains("# Reunion Test"));
    assert!(md.contains("Speaker 1"));
    assert!(md.contains("Bonjour"));
}
```

**Step 2: Run test, verify fail**

**Step 3: Implement exports**

- `export_markdown(title, segments, summary) -> String`
- `export_pdf(title, segments, summary, output_path)` - generate PDF from Markdown (use `printpdf` or shell out to `pandoc`)
- `export_notion(title, segments, summary, notion_token, page_id)` - POST to Notion API
- `export_slack(title, summary, webhook_url)` - POST to Slack webhook

**Step 4: Run tests, verify pass**

**Step 5: Commit**

```bash
git add src-tauri/src/export/
git commit -m "feat: implement export to Markdown, PDF, Notion, Slack"
```

---

## Phase 9: System Tray + Polish

### Task 19: Add system tray (menu bar)

**Files:**
- Modify: `src-tauri/src/main.rs`
- Create: `src-tauri/icons/tray-icon.png`

**Step 1: Configure Tauri system tray**

```rust
use tauri::{
    tray::{TrayIconBuilder, MouseButton, MouseButtonState},
    Manager,
};

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                })
                .build(app)?;
            Ok(())
        })
        // ... rest of builder
}
```

**Step 2: Add tray menu**

Right-click menu with: "Open PopTranscribe", "Quick Start (Visio)", "Quick Start (In-Person)", separator, "Quit".

**Step 3: Verify tray icon appears**

Run: `npm run tauri dev`
Expected: Icon in menu bar, click opens window.

**Step 4: Commit**

```bash
git add src-tauri/
git commit -m "feat: add system tray with menu"
```

---

### Task 20: Settings view + API key management

**Files:**
- Modify: `src/views/SettingsView.tsx`
- Modify: `src-tauri/src/commands.rs`

**Step 1: Build SettingsView**

- API key input (masked, stored securely via Tauri's secure storage or keychain)
- Default language selection
- Default capture mode
- Notion integration (API token + page ID)
- Slack webhook URL

**Step 2: Implement save/load settings in Rust**

Store settings in SQLite `settings` table (key-value). API key stored via macOS Keychain (use `security-framework` crate).

**Step 3: Commit**

```bash
git add src/ src-tauri/
git commit -m "feat: implement settings view with API key management"
```

---

## Phase 10: Integration Testing

### Task 21: End-to-end test

**Step 1: Manual test checklist**

- [ ] App launches from menu bar
- [ ] Start visio session -> transcription appears live
- [ ] Start in-person session -> transcription appears live
- [ ] Text search works during live session
- [ ] LLM search works during live session
- [ ] Stop session -> batch processing starts
- [ ] Notification when processing complete
- [ ] Detail view shows diarized transcript
- [ ] Speaker renaming works
- [ ] Summary is generated (key points, decisions, actions)
- [ ] Export Markdown works
- [ ] Export PDF works
- [ ] Settings save and persist
- [ ] App quits cleanly from tray

**Step 2: Fix any issues found**

**Step 3: Final commit**

```bash
git add -A
git commit -m "fix: address integration test issues"
```

---

## Summary

| Phase | Tasks | Description |
|---|---|---|
| 1 | 1-2 | Project scaffolding + module structure |
| 2 | 3-6 | Audio capture (mic + system + mixer + WAV) |
| 3 | 7-9 | Mistral API clients (batch + SSE + chat) |
| 4 | 10 | SQLite database |
| 5 | 11 | Tauri commands bridge |
| 6 | 12-15 | React frontend (4 views) |
| 7 | 16-17 | Full pipeline wiring + search |
| 8 | 18 | Export (MD, PDF, Notion, Slack) |
| 9 | 19-20 | System tray + settings |
| 10 | 21 | Integration testing |

**Total: 21 tasks, 10 phases.**
