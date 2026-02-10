use std::sync::Mutex;
use crate::db::Database;

pub struct AppState {
    pub db: Mutex<Database>,
    pub api_key: Mutex<String>,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        Self {
            db: Mutex::new(db),
            api_key: Mutex::new(String::new()),
        }
    }
}
