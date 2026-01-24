use std::sync::Arc;

use super::error::Result;
use super::executor;

/// Describes a service operation for command string display
#[derive(Debug, Clone)]
pub enum ServiceOp {
    CreateUser {
        username: String,
        groups: Vec<String>,
        shell: String,
    },
    SetLocale {
        locale: String,
    },
    SetKeymap {
        keymap: String,
    },
    SetTimezone {
        timezone: String,
    },
    RunCommand {
        cmd: Vec<String>,
    },
    RunCommandSudo {
        cmd: Vec<String>,
    },
}

/// Trait abstracting system operations for onboard setup
pub trait OnboardService: Send + Sync {
    // Query methods
    fn check_network(&self) -> bool;
    fn list_locales(&self) -> Vec<String>;
    fn list_keymaps(&self) -> Vec<String>;
    fn list_timezones(&self) -> Vec<String>;

    // Mutating methods
    fn create_user(&self, username: &str, password: &str, groups: &[String], shell: &str) -> Result<()>;
    fn set_locale(&self, locale: &str) -> Result<()>;
    fn set_keymap(&self, keymap: &str) -> Result<()>;
    fn set_timezone(&self, timezone: &str) -> Result<()>;
    fn run_command_as_user(&self, username: &str, cmd: &[String]) -> Result<String>;
    fn run_command_as_user_with_sudo(&self, username: &str, cmd: &[String], password: &str) -> Result<String>;
    fn remove_initial_session(&self) -> Result<()>;

    // Command description for display
    fn command_string(&self, op: &ServiceOp) -> String;
}

/// Format the command string for a given operation
fn format_command_string(op: &ServiceOp) -> String {
    match op {
        ServiceOp::CreateUser { username, groups, shell } => {
            let mut parts = vec!["useradd".to_string(), "-m".to_string(), "-s".to_string(), shell.clone()];
            if !groups.is_empty() {
                parts.push("-G".to_string());
                parts.push(groups.join(","));
            }
            parts.push(username.clone());
            parts.join(" ")
        }
        ServiceOp::SetLocale { locale } => {
            format!("localectl set-locale LANG={locale}")
        }
        ServiceOp::SetKeymap { keymap } => {
            format!("localectl set-keymap {keymap}")
        }
        ServiceOp::SetTimezone { timezone } => {
            format!("timedatectl set-timezone {timezone}")
        }
        ServiceOp::RunCommand { cmd } => {
            cmd.join(" ")
        }
        ServiceOp::RunCommandSudo { cmd } => {
            format!("sudo {}", cmd.join(" "))
        }
    }
}

/// Live service that executes real system commands
pub struct LiveService;

impl OnboardService for LiveService {
    fn check_network(&self) -> bool {
        executor::check_network(false)
    }

    fn list_locales(&self) -> Vec<String> {
        executor::list_locales(false)
    }

    fn list_keymaps(&self) -> Vec<String> {
        executor::list_keymaps(false)
    }

    fn list_timezones(&self) -> Vec<String> {
        executor::list_timezones(false)
    }

    fn create_user(&self, username: &str, password: &str, groups: &[String], shell: &str) -> Result<()> {
        executor::create_user(username, password, groups, shell)
    }

    fn set_locale(&self, locale: &str) -> Result<()> {
        executor::set_locale(locale)
    }

    fn set_keymap(&self, keymap: &str) -> Result<()> {
        executor::set_keymap(keymap)
    }

    fn set_timezone(&self, timezone: &str) -> Result<()> {
        executor::set_timezone(timezone)
    }

    fn run_command_as_user(&self, username: &str, cmd: &[String]) -> Result<String> {
        executor::run_command_as_user(username, cmd)
    }

    fn run_command_as_user_with_sudo(&self, username: &str, cmd: &[String], password: &str) -> Result<String> {
        executor::run_command_as_user_with_sudo(username, cmd, password)
    }

    fn remove_initial_session(&self) -> Result<()> {
        executor::remove_initial_session()
    }

    fn command_string(&self, op: &ServiceOp) -> String {
        format_command_string(op)
    }
}

/// Dryrun service that simulates operations without system changes
pub struct DryrunService;

impl OnboardService for DryrunService {
    fn check_network(&self) -> bool {
        executor::check_network(true)
    }

    fn list_locales(&self) -> Vec<String> {
        executor::list_locales(true)
    }

    fn list_keymaps(&self) -> Vec<String> {
        executor::list_keymaps(true)
    }

    fn list_timezones(&self) -> Vec<String> {
        executor::list_timezones(true)
    }

    fn create_user(&self, _username: &str, _password: &str, _groups: &[String], _shell: &str) -> Result<()> {
        Ok(())
    }

    fn set_locale(&self, _locale: &str) -> Result<()> {
        Ok(())
    }

    fn set_keymap(&self, _keymap: &str) -> Result<()> {
        Ok(())
    }

    fn set_timezone(&self, _timezone: &str) -> Result<()> {
        Ok(())
    }

    fn run_command_as_user(&self, _username: &str, _cmd: &[String]) -> Result<String> {
        Ok(String::new())
    }

    fn run_command_as_user_with_sudo(&self, _username: &str, _cmd: &[String], _password: &str) -> Result<String> {
        Ok(String::new())
    }

    fn remove_initial_session(&self) -> Result<()> {
        Ok(())
    }

    fn command_string(&self, op: &ServiceOp) -> String {
        format_command_string(op)
    }
}

/// Create the appropriate service based on dryrun mode
pub fn create_service(dryrun: bool) -> Arc<dyn OnboardService> {
    if dryrun {
        Arc::new(DryrunService)
    } else {
        Arc::new(LiveService)
    }
}
