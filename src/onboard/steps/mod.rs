/// Unique identifier for each wizard step
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepId {
    User,
    Locale,
    Keyboard,
    Network,
    Preferences,
    Review,
    Update,
    Reboot,
}

impl StepId {
    pub fn short_name(&self) -> &'static str {
        match self {
            StepId::Network => "Network",
            StepId::User => "User",
            StepId::Locale => "Locale",
            StepId::Keyboard => "Keyboard",
            StepId::Preferences => "Prefs",
            StepId::Review => "Review",
            StepId::Update => "Update",
            StepId::Reboot => "Reboot",
        }
    }
}

/// Result of completing a step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StepResult {
    #[default]
    Pending,
    Completed,
    Skipped,
    Failed,
    /// Step is locked until prerequisite completes
    Locked,
}
