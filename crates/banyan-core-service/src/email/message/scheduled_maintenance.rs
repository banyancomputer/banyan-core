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
    const SUBJECT: &'static str = "Scheduled Maintenance";
    const TEMPLATE_NAME: &'static str = "scheduled_maintenance";
    const TYPE_NAME: &'static str = "scheduled_maintenance";
}
