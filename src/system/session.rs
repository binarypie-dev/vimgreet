use ini::Ini;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct Session {
    pub name: String,
    pub slug: String,
    pub exec: String,
    pub desktop_names: Vec<String>,
    pub session_type: SessionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SessionType {
    #[default]
    Wayland,
    X11,
}

impl SessionType {
    pub fn as_xdg_type(&self) -> &'static str {
        match self {
            SessionType::Wayland => "wayland",
            SessionType::X11 => "x11",
        }
    }
}

pub fn discover_sessions() -> Vec<Session> {
    let mut sessions = Vec::new();

    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());

    for dir in data_dirs.split(':') {
        let wayland_path = PathBuf::from(dir).join("wayland-sessions");
        if wayland_path.exists() {
            sessions.extend(load_sessions_from_dir(&wayland_path, SessionType::Wayland));
        }

        let x11_path = PathBuf::from(dir).join("xsessions");
        if x11_path.exists() {
            sessions.extend(load_sessions_from_dir(&x11_path, SessionType::X11));
        }
    }

    sessions.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    sessions.dedup_by(|a, b| a.slug == b.slug);

    debug!("Discovered {} sessions", sessions.len());
    sessions
}

fn load_sessions_from_dir(dir: &PathBuf, session_type: SessionType) -> Vec<Session> {
    let mut sessions = Vec::new();

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            warn!("Failed to read session directory {:?}: {}", dir, e);
            return sessions;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "desktop") {
            if let Some(session) = parse_desktop_file(&path, session_type) {
                sessions.push(session);
            }
        }
    }

    sessions
}

fn parse_desktop_file(path: &Path, session_type: SessionType) -> Option<Session> {
    let ini = Ini::load_from_file(path).ok()?;
    let section = ini.section(Some("Desktop Entry"))?;

    if section.get("Hidden") == Some("true") || section.get("NoDisplay") == Some("true") {
        return None;
    }

    let name = section.get("Name")?.to_string();
    let exec = section.get("Exec")?.to_string();

    let slug = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())?;

    let desktop_names = section
        .get("DesktopNames")
        .map(|s| s.split(';').map(|s| s.to_string()).collect())
        .unwrap_or_default();

    Some(Session {
        name,
        slug,
        exec,
        desktop_names,
        session_type,
    })
}

impl Session {
    pub fn build_env(&self) -> Vec<String> {
        let mut env = vec![
            format!("XDG_SESSION_TYPE={}", self.session_type.as_xdg_type()),
        ];

        if !self.desktop_names.is_empty() {
            env.push(format!(
                "XDG_CURRENT_DESKTOP={}",
                self.desktop_names.join(":")
            ));
        }

        env
    }

    pub fn build_cmd(&self) -> Vec<String> {
        shell_words::split(&self.exec).unwrap_or_else(|_| vec![self.exec.clone()])
    }
}
