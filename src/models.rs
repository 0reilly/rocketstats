use crate::schema::event_tracking;
use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Queryable)]
pub struct EventData {
    pub id: i32,
    pub url: String,
    pub referrer: String,
    pub user_agent: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Insertable)]
#[table_name = "event_tracking"]
pub struct NewEventData {
    pub url: String,
    pub referrer: String,
    pub user_agent: String,
    pub timestamp: DateTime<Utc>,
}
