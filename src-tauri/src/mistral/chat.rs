use serde::{Serialize, Deserialize};

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
