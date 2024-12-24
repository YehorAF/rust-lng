use chrono::{DateTime, Utc};

#[derive(Default, Clone)]
pub struct Task {
    pub id: u64,
    pub name: String,
    pub is_completed: bool,
    pub create_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}