use serde::Serialize;

use super::EmailMessage;

#[derive(Serialize)]
pub struct ScheduledMaintenance {
    /// The start time of the maintenance window
    pub(crate) start: String,
    /// The end time of the maintenance window
    pub(crate) end: String,
}

impl EmailMessage for ScheduledMaintenance {
    fn subject() -> String {
        "Scheduled Maintenance".to_string()
    }

    fn template_name() -> &'static str {
        "scheduled_maintenance"
    }
} 