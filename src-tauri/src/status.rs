use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PaneStatusKind {
    Idle,
    Sending,
    Ok,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaneStatusEvent {
    pub id: String,
    pub status: PaneStatusKind,
    pub message: Option<String>,
}

pub const EVENT_PANE_STATUS: &str = "pane_status";
