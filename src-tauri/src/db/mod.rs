use rusqlite::{Connection, params};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub mode: String,
    pub audio_path: Option<String>,
    pub created_at: String,
    pub duration_secs: Option<f64>,
    pub summary_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: i64,
    pub session_id: String,
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
    pub speaker: Option<String>,
    pub is_diarized: bool,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &std::path::Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    pub fn new_in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                mode TEXT NOT NULL,
                audio_path TEXT,
                created_at TEXT NOT NULL,
                duration_secs REAL,
                summary_json TEXT
            );

            CREATE TABLE IF NOT EXISTS segments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL REFERENCES sessions(id),
                text TEXT NOT NULL,
                start_time REAL NOT NULL,
                end_time REAL NOT NULL,
                speaker TEXT,
                is_diarized INTEGER NOT NULL DEFAULT 0
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS segments_fts USING fts5(text, content=segments, content_rowid=id);

            CREATE TRIGGER IF NOT EXISTS segments_ai AFTER INSERT ON segments BEGIN
                INSERT INTO segments_fts(rowid, text) VALUES (new.id, new.text);
            END;

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );"
        )?;
        Ok(())
    }

    // ── Sessions ──────────────────────────────────────────────────────

    pub fn create_session(&self, title: &str, mode: &str) -> Result<String, rusqlite::Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO sessions (id, title, mode, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, title, mode, now],
        )?;
        Ok(id)
    }

    pub fn get_session(&self, id: &str) -> Result<Session, rusqlite::Error> {
        self.conn.query_row(
            "SELECT id, title, mode, audio_path, created_at, duration_secs, summary_json
             FROM sessions WHERE id = ?1",
            params![id],
            |row| {
                Ok(Session {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    mode: row.get(2)?,
                    audio_path: row.get(3)?,
                    created_at: row.get(4)?,
                    duration_secs: row.get(5)?,
                    summary_json: row.get(6)?,
                })
            },
        )
    }

    pub fn list_sessions(&self) -> Result<Vec<Session>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, mode, audio_path, created_at, duration_secs, summary_json
             FROM sessions ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                title: row.get(1)?,
                mode: row.get(2)?,
                audio_path: row.get(3)?,
                created_at: row.get(4)?,
                duration_secs: row.get(5)?,
                summary_json: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    pub fn update_session_duration(&self, id: &str, duration_secs: f64) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE sessions SET duration_secs = ?1 WHERE id = ?2",
            params![duration_secs, id],
        )?;
        Ok(())
    }

    pub fn update_session_audio_path(&self, id: &str, path: &str) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE sessions SET audio_path = ?1 WHERE id = ?2",
            params![path, id],
        )?;
        Ok(())
    }

    pub fn update_session_title(&self, id: &str, title: &str) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE sessions SET title = ?1 WHERE id = ?2",
            params![title, id],
        )?;
        Ok(())
    }

    pub fn save_summary(&self, session_id: &str, summary_json: &str) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE sessions SET summary_json = ?1 WHERE id = ?2",
            params![summary_json, session_id],
        )?;
        Ok(())
    }

    // ── Segments ──────────────────────────────────────────────────────

    pub fn save_segment(
        &self,
        session_id: &str,
        text: &str,
        start: f64,
        end: f64,
        speaker: Option<&str>,
        is_diarized: bool,
    ) -> Result<i64, rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO segments (session_id, text, start_time, end_time, speaker, is_diarized)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![session_id, text, start, end, speaker, is_diarized as i32],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_segments(&self, session_id: &str) -> Result<Vec<Segment>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, text, start_time, end_time, speaker, is_diarized
             FROM segments WHERE session_id = ?1 ORDER BY start_time ASC",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            let is_diarized_int: i32 = row.get(6)?;
            Ok(Segment {
                id: row.get(0)?,
                session_id: row.get(1)?,
                text: row.get(2)?,
                start_time: row.get(3)?,
                end_time: row.get(4)?,
                speaker: row.get(5)?,
                is_diarized: is_diarized_int != 0,
            })
        })?;
        rows.collect()
    }

    pub fn clear_segments(&self, session_id: &str, diarized_only: bool) -> Result<(), rusqlite::Error> {
        if diarized_only {
            self.conn.execute(
                "DELETE FROM segments WHERE session_id = ?1 AND is_diarized = 1",
                params![session_id],
            )?;
        } else {
            self.conn.execute(
                "DELETE FROM segments WHERE session_id = ?1",
                params![session_id],
            )?;
        }
        Ok(())
    }

    pub fn rename_speaker(
        &self,
        session_id: &str,
        old_name: &str,
        new_name: &str,
    ) -> Result<usize, rusqlite::Error> {
        let changed = self.conn.execute(
            "UPDATE segments SET speaker = ?1 WHERE session_id = ?2 AND speaker = ?3",
            params![new_name, session_id, old_name],
        )?;
        Ok(changed)
    }

    pub fn search_text(
        &self,
        query: &str,
        session_id: Option<&str>,
    ) -> Result<Vec<Segment>, rusqlite::Error> {
        fn row_to_segment(row: &rusqlite::Row<'_>) -> Result<Segment, rusqlite::Error> {
            let is_diarized_int: i32 = row.get(6)?;
            Ok(Segment {
                id: row.get(0)?,
                session_id: row.get(1)?,
                text: row.get(2)?,
                start_time: row.get(3)?,
                end_time: row.get(4)?,
                speaker: row.get(5)?,
                is_diarized: is_diarized_int != 0,
            })
        }

        if let Some(sid) = session_id {
            let mut stmt = self.conn.prepare(
                "SELECT s.id, s.session_id, s.text, s.start_time, s.end_time, s.speaker, s.is_diarized
                 FROM segments s
                 INNER JOIN segments_fts fts ON s.id = fts.rowid
                 WHERE segments_fts MATCH ?1 AND s.session_id = ?2
                 ORDER BY s.start_time ASC",
            )?;
            let rows = stmt.query_map(params![query, sid], |row| row_to_segment(row))?;
            rows.collect()
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT s.id, s.session_id, s.text, s.start_time, s.end_time, s.speaker, s.is_diarized
                 FROM segments s
                 INNER JOIN segments_fts fts ON s.id = fts.rowid
                 WHERE segments_fts MATCH ?1
                 ORDER BY s.start_time ASC",
            )?;
            let rows = stmt.query_map(params![query], |row| row_to_segment(row))?;
            rows.collect()
        }
    }

    // ── Settings ──────────────────────────────────────────────────────

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT value FROM settings WHERE key = ?1",
        )?;
        let mut rows = stmt.query_map(params![key], |row| row.get::<_, String>(0))?;
        match rows.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get_session() {
        let db = Database::new_in_memory().unwrap();
        let id = db.create_session("Test Meeting", "visio").unwrap();
        let session = db.get_session(&id).unwrap();
        assert_eq!(session.title, "Test Meeting");
        assert_eq!(session.mode, "visio");
        assert!(session.audio_path.is_none());
        assert!(session.duration_secs.is_none());
        assert!(session.summary_json.is_none());
    }

    #[test]
    fn test_list_sessions() {
        let db = Database::new_in_memory().unwrap();
        db.create_session("Meeting 1", "visio").unwrap();
        db.create_session("Meeting 2", "inperson").unwrap();
        let sessions = db.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_update_session_duration() {
        let db = Database::new_in_memory().unwrap();
        let id = db.create_session("Test", "visio").unwrap();
        db.update_session_duration(&id, 120.5).unwrap();
        let session = db.get_session(&id).unwrap();
        assert_eq!(session.duration_secs, Some(120.5));
    }

    #[test]
    fn test_update_session_audio_path() {
        let db = Database::new_in_memory().unwrap();
        let id = db.create_session("Test", "visio").unwrap();
        db.update_session_audio_path(&id, "/tmp/audio.wav").unwrap();
        let session = db.get_session(&id).unwrap();
        assert_eq!(session.audio_path.as_deref(), Some("/tmp/audio.wav"));
    }

    #[test]
    fn test_update_session_title() {
        let db = Database::new_in_memory().unwrap();
        let id = db.create_session("Old Title", "visio").unwrap();
        db.update_session_title(&id, "New Title").unwrap();
        let session = db.get_session(&id).unwrap();
        assert_eq!(session.title, "New Title");
    }

    #[test]
    fn test_save_summary() {
        let db = Database::new_in_memory().unwrap();
        let id = db.create_session("Test", "visio").unwrap();
        let summary = r#"{"key_points": ["point 1"]}"#;
        db.save_summary(&id, summary).unwrap();
        let session = db.get_session(&id).unwrap();
        assert_eq!(session.summary_json.as_deref(), Some(summary));
    }

    #[test]
    fn test_save_and_get_segments() {
        let db = Database::new_in_memory().unwrap();
        let id = db.create_session("Test", "visio").unwrap();
        db.save_segment(&id, "Bonjour", 0.0, 1.5, None, false).unwrap();
        db.save_segment(&id, "Comment ca va", 1.5, 3.0, Some("Speaker 1"), true).unwrap();
        let segments = db.get_segments(&id).unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].text, "Bonjour");
        assert!(!segments[0].is_diarized);
        assert!(segments[0].speaker.is_none());
        assert_eq!(segments[1].text, "Comment ca va");
        assert_eq!(segments[1].speaker.as_deref(), Some("Speaker 1"));
        assert!(segments[1].is_diarized);
    }

    #[test]
    fn test_clear_segments_all() {
        let db = Database::new_in_memory().unwrap();
        let id = db.create_session("Test", "visio").unwrap();
        db.save_segment(&id, "Seg 1", 0.0, 1.0, None, false).unwrap();
        db.save_segment(&id, "Seg 2", 1.0, 2.0, Some("S1"), true).unwrap();
        db.clear_segments(&id, false).unwrap();
        let segments = db.get_segments(&id).unwrap();
        assert_eq!(segments.len(), 0);
    }

    #[test]
    fn test_clear_segments_diarized_only() {
        let db = Database::new_in_memory().unwrap();
        let id = db.create_session("Test", "visio").unwrap();
        db.save_segment(&id, "Seg 1", 0.0, 1.0, None, false).unwrap();
        db.save_segment(&id, "Seg 2", 1.0, 2.0, Some("S1"), true).unwrap();
        db.clear_segments(&id, true).unwrap();
        let segments = db.get_segments(&id).unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "Seg 1");
    }

    #[test]
    fn test_search_text() {
        let db = Database::new_in_memory().unwrap();
        let id = db.create_session("Test", "visio").unwrap();
        db.save_segment(&id, "Discussion sur le budget", 0.0, 2.0, None, false).unwrap();
        db.save_segment(&id, "Le planning est ok", 2.0, 4.0, None, false).unwrap();
        let results = db.search_text("budget", None).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].text.contains("budget"));
    }

    #[test]
    fn test_search_text_with_session_filter() {
        let db = Database::new_in_memory().unwrap();
        let id1 = db.create_session("Session 1", "visio").unwrap();
        let id2 = db.create_session("Session 2", "visio").unwrap();
        db.save_segment(&id1, "Budget discussion", 0.0, 2.0, None, false).unwrap();
        db.save_segment(&id2, "Budget review", 0.0, 2.0, None, false).unwrap();
        let results = db.search_text("budget", Some(&id1)).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].session_id, id1);
    }

    #[test]
    fn test_rename_speaker() {
        let db = Database::new_in_memory().unwrap();
        let id = db.create_session("Test", "visio").unwrap();
        db.save_segment(&id, "Hello", 0.0, 1.0, Some("Speaker 1"), true).unwrap();
        db.save_segment(&id, "Hi", 1.0, 2.0, Some("Speaker 1"), true).unwrap();
        db.save_segment(&id, "Hey", 2.0, 3.0, Some("Speaker 2"), true).unwrap();
        let count = db.rename_speaker(&id, "Speaker 1", "Alexandre").unwrap();
        assert_eq!(count, 2);
        let segments = db.get_segments(&id).unwrap();
        assert_eq!(segments[0].speaker.as_deref(), Some("Alexandre"));
        assert_eq!(segments[1].speaker.as_deref(), Some("Alexandre"));
        assert_eq!(segments[2].speaker.as_deref(), Some("Speaker 2"));
    }

    #[test]
    fn test_settings() {
        let db = Database::new_in_memory().unwrap();
        assert!(db.get_setting("api_key").unwrap().is_none());
        db.set_setting("api_key", "test-key").unwrap();
        assert_eq!(db.get_setting("api_key").unwrap().unwrap(), "test-key");
        db.set_setting("api_key", "new-key").unwrap();
        assert_eq!(db.get_setting("api_key").unwrap().unwrap(), "new-key");
    }

    #[test]
    fn test_get_session_not_found() {
        let db = Database::new_in_memory().unwrap();
        let result = db.get_session("nonexistent-id");
        assert!(result.is_err());
    }
}
