/// Dynamic status bar state that can be updated by content panels
#[derive(Debug, Clone, Default)]
pub struct StatusBarState {
    /// Left side hint text (e.g., "i: insert  j/k: fields")
    pub left_hint: String,
    /// Right side hint text (e.g., "Ctrl+h: sidebar")
    pub right_hint: String,
}

impl StatusBarState {
    /// Get hints for normal mode in sidebar
    pub fn sidebar_normal() -> Self {
        Self {
            left_hint: "j/k: navigate".to_string(),
            right_hint: "l/Enter: edit  :help".to_string(),
        }
    }

    /// Get hints for normal mode in content with picker
    pub fn content_picker_normal() -> Self {
        Self {
            left_hint: "j/k: navigate  Enter: select".to_string(),
            right_hint: "i: filter  Ctrl+h: sidebar".to_string(),
        }
    }

    /// Get hints for insert mode in content with picker
    pub fn content_picker_insert() -> Self {
        Self {
            left_hint: "Type to filter".to_string(),
            right_hint: "Esc: normal  Enter: select".to_string(),
        }
    }

    /// Get hints for normal mode in content with form
    pub fn content_form_normal() -> Self {
        Self {
            left_hint: "j/k: fields  i: edit".to_string(),
            right_hint: "Enter: submit  Ctrl+h: sidebar".to_string(),
        }
    }

    /// Get hints for insert mode in content with form
    pub fn content_form_insert() -> Self {
        Self {
            left_hint: "Type to enter text".to_string(),
            right_hint: "Esc: normal  Tab: next field".to_string(),
        }
    }

    /// Get hints for command mode
    pub fn command_mode() -> Self {
        Self {
            left_hint: String::new(),
            right_hint: "Enter: run  Esc: cancel".to_string(),
        }
    }

    /// Get hints for welcome screen
    pub fn welcome() -> Self {
        Self {
            left_hint: String::new(),
            right_hint: "Enter: start setup".to_string(),
        }
    }

    /// Get hints for review step
    pub fn review_step() -> Self {
        Self {
            left_hint: "Review your settings".to_string(),
            right_hint: "Enter: apply  Ctrl+h: sidebar".to_string(),
        }
    }

    /// Get hints for update step
    pub fn update_step(needs_password: bool) -> Self {
        if needs_password {
            Self {
                left_hint: "Password required".to_string(),
                right_hint: "i: enter password  :skip".to_string(),
            }
        } else {
            Self {
                left_hint: "Ready to run commands".to_string(),
                right_hint: "Enter: run  :skip".to_string(),
            }
        }
    }

    /// Get hints for reboot/finish step
    pub fn reboot_step() -> Self {
        Self {
            left_hint: "Setup complete!".to_string(),
            right_hint: "Enter: reboot system".to_string(),
        }
    }

    /// Get hints for network step
    pub fn network_step(connected: bool) -> Self {
        if connected {
            Self {
                left_hint: "Network connected".to_string(),
                right_hint: "Enter: next  Ctrl+h: sidebar".to_string(),
            }
        } else {
            Self {
                left_hint: "Network not connected".to_string(),
                right_hint: "Enter: configure  :skip".to_string(),
            }
        }
    }

    /// Get hints when a step is locked
    pub fn locked_step() -> Self {
        Self {
            left_hint: "Step locked".to_string(),
            right_hint: "Complete previous steps first".to_string(),
        }
    }

    /// Get hints while executing
    pub fn executing() -> Self {
        Self {
            left_hint: "Please wait...".to_string(),
            right_hint: String::new(),
        }
    }
}
