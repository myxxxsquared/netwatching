use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatMsg {
    pub name: String,
    pub sending_time: DateTime<Utc>,
}
