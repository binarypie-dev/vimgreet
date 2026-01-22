use std::io::Write;
use std::process::{Command, Stdio};
use tracing::{debug, info, warn};

use super::error::{OnboardError, Result};

/// Check if network is connected by testing DNS resolution
pub fn check_network(demo_mode: bool) -> bool {
    if demo_mode {
        return true;
    }

    // Try to resolve a well-known hostname
    Command::new("ping")
        .args(["-c", "1", "-W", "2", "1.1.1.1"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// List available locales from the system
pub fn list_locales(demo_mode: bool) -> Vec<String> {
    if demo_mode {
        return vec![
            "en_US.UTF-8".to_string(),
            "en_GB.UTF-8".to_string(),
            "de_DE.UTF-8".to_string(),
            "fr_FR.UTF-8".to_string(),
            "es_ES.UTF-8".to_string(),
            "it_IT.UTF-8".to_string(),
            "pt_BR.UTF-8".to_string(),
            "ja_JP.UTF-8".to_string(),
            "zh_CN.UTF-8".to_string(),
            "ko_KR.UTF-8".to_string(),
        ];
    }

    let output = Command::new("localectl")
        .arg("list-locales")
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect()
        }
        _ => {
            warn!("Failed to list locales, using fallback");
            vec!["en_US.UTF-8".to_string()]
        }
    }
}

/// Set the system locale
pub fn set_locale(locale: &str) -> Result<()> {
    info!("Setting locale to: {}", locale);

    let status = Command::new("localectl")
        .args(["set-locale", &format!("LANG={}", locale)])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(OnboardError::Command(format!(
            "localectl set-locale failed with code {:?}",
            status.code()
        )))
    }
}

/// List available keyboard layouts
pub fn list_keymaps(demo_mode: bool) -> Vec<String> {
    if demo_mode {
        return vec![
            "us".to_string(),
            "uk".to_string(),
            "de".to_string(),
            "fr".to_string(),
            "es".to_string(),
            "it".to_string(),
            "pt".to_string(),
            "ru".to_string(),
            "jp".to_string(),
            "cn".to_string(),
            "dvorak".to_string(),
            "colemak".to_string(),
        ];
    }

    let output = Command::new("localectl")
        .arg("list-keymaps")
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect()
        }
        _ => {
            warn!("Failed to list keymaps, using fallback");
            vec!["us".to_string()]
        }
    }
}

/// Set the keyboard layout
pub fn set_keymap(keymap: &str) -> Result<()> {
    info!("Setting keymap to: {}", keymap);

    let status = Command::new("localectl")
        .args(["set-keymap", keymap])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(OnboardError::Command(format!(
            "localectl set-keymap failed with code {:?}",
            status.code()
        )))
    }
}

/// List available timezones
pub fn list_timezones(demo_mode: bool) -> Vec<String> {
    if demo_mode {
        return vec![
            "UTC".to_string(),
            "America/New_York".to_string(),
            "America/Chicago".to_string(),
            "America/Denver".to_string(),
            "America/Los_Angeles".to_string(),
            "America/Sao_Paulo".to_string(),
            "Europe/London".to_string(),
            "Europe/Paris".to_string(),
            "Europe/Berlin".to_string(),
            "Asia/Tokyo".to_string(),
            "Asia/Shanghai".to_string(),
            "Asia/Kolkata".to_string(),
            "Australia/Sydney".to_string(),
        ];
    }

    let output = Command::new("timedatectl")
        .args(["list-timezones"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect()
        }
        _ => {
            warn!("Failed to list timezones, using fallback");
            vec!["UTC".to_string()]
        }
    }
}

/// Set the system timezone
pub fn set_timezone(timezone: &str) -> Result<()> {
    info!("Setting timezone to: {}", timezone);

    let status = Command::new("timedatectl")
        .args(["set-timezone", timezone])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(OnboardError::Command(format!(
            "timedatectl set-timezone failed with code {:?}",
            status.code()
        )))
    }
}

/// Create a new user account
pub fn create_user(username: &str, password: &str, groups: &[String], shell: &str) -> Result<()> {
    info!("Creating user: {}", username);

    // Build useradd command
    let mut args = vec![
        "-m".to_string(),  // Create home directory
        "-s".to_string(),
        shell.to_string(),
    ];

    if !groups.is_empty() {
        args.push("-G".to_string());
        args.push(groups.join(","));
    }

    args.push(username.to_string());

    debug!("Running: useradd {:?}", args);

    let status = Command::new("useradd")
        .args(&args)
        .status()?;

    if !status.success() {
        return Err(OnboardError::UserCreation(format!(
            "useradd failed with code {:?}",
            status.code()
        )));
    }

    // Set password via chpasswd
    info!("Setting password for user: {}", username);

    let mut child = Command::new("chpasswd")
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "{}:{}", username, password)?;
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(OnboardError::UserCreation(format!(
            "chpasswd failed with code {:?}",
            status.code()
        )));
    }

    info!("User {} created successfully", username);
    Ok(())
}

/// Run a command as a specific user (using su)
pub fn run_command_as_user(username: &str, cmd: &[String]) -> Result<String> {
    if cmd.is_empty() {
        return Err(OnboardError::Command("Empty command".to_string()));
    }

    info!("Running command as {}: {:?}", username, cmd);

    // Use su to run command as user
    // su -l username -c "command args..."
    let command_str = cmd.iter()
        .map(|s| shell_escape::escape(s.into()).to_string())
        .collect::<Vec<_>>()
        .join(" ");

    let output = Command::new("su")
        .args(["-l", username, "-c", &command_str])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(OnboardError::Command(format!(
            "Command failed: {}",
            if stderr.is_empty() { stdout } else { stderr }
        )))
    }
}

/// Run a command as a specific user with sudo (provides password via stdin)
pub fn run_command_as_user_with_sudo(username: &str, cmd: &[String], password: &str) -> Result<String> {
    if cmd.is_empty() {
        return Err(OnboardError::Command("Empty command".to_string()));
    }

    info!("Running sudo command as {}: {:?}", username, cmd);

    // Build the command string - the first element should be the command, rest are args
    // We need to run: su -l username -c "echo password | sudo -S command args..."
    let command_str = cmd.iter()
        .map(|s| shell_escape::escape(s.into()).to_string())
        .collect::<Vec<_>>()
        .join(" ");

    // Use -S flag to read password from stdin
    let sudo_cmd = format!("echo {} | sudo -S {}", shell_escape::escape(password.into()), command_str);

    let output = Command::new("su")
        .args(["-l", username, "-c", &sudo_cmd])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        // Filter out password prompt from stderr
        let filtered_stderr = stderr
            .lines()
            .filter(|line| !line.contains("[sudo]") && !line.contains("password"))
            .collect::<Vec<_>>()
            .join("\n");

        Err(OnboardError::Command(format!(
            "Command failed: {}",
            if filtered_stderr.is_empty() { stdout } else { filtered_stderr }
        )))
    }
}

/// Remove the initial_session block from greetd config
pub fn remove_initial_session() -> Result<()> {
    const GREETD_CONFIG: &str = "/etc/greetd/config.toml";

    info!("Removing initial_session from greetd config");

    let content = std::fs::read_to_string(GREETD_CONFIG)?;
    let mut lines: Vec<&str> = content.lines().collect();

    // Find and remove [initial_session] block
    let mut in_initial_session = false;
    let mut to_remove = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == "[initial_session]" {
            in_initial_session = true;
            to_remove.push(i);
        } else if in_initial_session {
            if trimmed.starts_with('[') {
                // New section started
                in_initial_session = false;
            } else {
                to_remove.push(i);
            }
        }
    }

    // Remove lines in reverse order to preserve indices
    for i in to_remove.into_iter().rev() {
        lines.remove(i);
    }

    // Write back
    let new_content = lines.join("\n");
    std::fs::write(GREETD_CONFIG, new_content)?;

    info!("Successfully removed initial_session from greetd config");
    Ok(())
}
