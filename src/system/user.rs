use std::fs;
use std::path::Path;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub display_name: Option<String>,
}

pub fn discover_users() -> Vec<User> {
    let (min_uid, max_uid) = read_uid_bounds();
    let mut users = Vec::new();

    // Read /etc/passwd directly for user enumeration
    if let Ok(content) = fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            if let Some(user) = parse_passwd_line(line, min_uid, max_uid) {
                if !should_hide_user(&user.username) {
                    users.push(user);
                }
            }
        }
    } else {
        warn!("Could not read /etc/passwd");
    }

    users.sort_by(|a, b| a.username.cmp(&b.username));
    debug!("Discovered {} users", users.len());
    users
}

fn parse_passwd_line(line: &str, min_uid: u32, max_uid: u32) -> Option<User> {
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() < 7 {
        return None;
    }

    let username = parts[0].to_string();
    let uid: u32 = parts[2].parse().ok()?;
    let gecos = parts[4];
    let shell = parts[6].to_string();

    if uid < min_uid || uid > max_uid {
        return None;
    }

    // Skip users with nologin shell
    if shell.contains("nologin") || shell.contains("false") {
        return None;
    }

    let display_name = {
        let name = gecos.split(',').next().unwrap_or("").trim();
        if name.is_empty() || name == username {
            None
        } else {
            Some(name.to_string())
        }
    };

    Some(User {
        username,
        display_name,
    })
}

fn read_uid_bounds() -> (u32, u32) {
    let login_defs = Path::new("/etc/login.defs");
    let mut min_uid = 1000u32;
    let mut max_uid = 60000u32;

    if let Ok(content) = fs::read_to_string(login_defs) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            let mut parts = line.split_whitespace();
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                match key {
                    "UID_MIN" => {
                        if let Ok(v) = value.parse() {
                            min_uid = v;
                        }
                    }
                    "UID_MAX" => {
                        if let Ok(v) = value.parse() {
                            max_uid = v;
                        }
                    }
                    _ => {}
                }
            }
        }
    } else {
        warn!("Could not read /etc/login.defs, using defaults");
    }

    debug!("UID bounds: {} - {}", min_uid, max_uid);
    (min_uid, max_uid)
}

fn should_hide_user(username: &str) -> bool {
    const HIDDEN_USERS: &[&str] = &["nobody", "nfsnobody", "greeter"];
    HIDDEN_USERS.contains(&username)
}
