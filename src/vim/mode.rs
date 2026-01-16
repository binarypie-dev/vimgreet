#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VimMode {
    #[default]
    Normal,
    Insert,
    Command,
}

impl VimMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            VimMode::Normal => "NORMAL",
            VimMode::Insert => "INSERT",
            VimMode::Command => "COMMAND",
        }
    }

    pub fn transition(&self, action: ModeAction) -> VimMode {
        match (self, action) {
            (VimMode::Normal, ModeAction::EnterInsert) => VimMode::Insert,
            (VimMode::Normal, ModeAction::EnterCommand) => VimMode::Command,
            (VimMode::Insert, ModeAction::Escape) => VimMode::Normal,
            (VimMode::Command, ModeAction::Escape) => VimMode::Normal,
            (VimMode::Command, ModeAction::Execute) => VimMode::Normal,
            _ => *self,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeAction {
    EnterInsert,
    EnterCommand,
    Escape,
    Execute,
}
