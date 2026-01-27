use serde::Deserialize;
use std::path::Path;
use tracing::info;

const DEFAULT_CONFIG_PATH: &str = "/etc/hypercube/onboard.toml";

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct OnboardConfig {
    pub general: GeneralConfig,
    pub network: NetworkConfig,
    pub user: UserConfig,
    pub locale: LocaleConfig,
    pub keyboard: KeyboardConfig,
    pub preferences: PreferencesConfig,
    pub completion: CompletionConfig,
    /// Update categories for the Update step
    #[serde(default)]
    pub updates: Vec<UpdateCategory>,
}

impl Default for OnboardConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            network: NetworkConfig::default(),
            user: UserConfig::default(),
            locale: LocaleConfig::default(),
            keyboard: KeyboardConfig::default(),
            preferences: PreferencesConfig::default(),
            completion: CompletionConfig::default(),
            updates: Vec::new(),
        }
    }
}

/// An update category containing packages to install
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCategory {
    /// Display name for the category
    pub name: String,
    /// Description shown to user
    #[serde(default)]
    pub description: String,
    /// Whether packages in this category are selected by default
    #[serde(default)]
    pub enabled_by_default: bool,
    /// Packages available in this category
    #[serde(default)]
    pub packages: Vec<PackageItem>,
}

/// A package that can be individually selected for installation
#[derive(Debug, Clone, Deserialize)]
pub struct PackageItem {
    /// Display title (e.g., "Firefox")
    pub title: String,
    /// Description shown to user (e.g., "Fast, private web browser")
    #[serde(default)]
    pub description: String,
    /// Whether this package is selected by default (overrides category default if set)
    pub enabled_by_default: Option<bool>,
    /// Whether this package is required and cannot be deselected
    #[serde(default)]
    pub required: bool,
    /// Commands to install this package
    pub commands: Vec<CommandConfig>,
}

impl PackageItem {
    /// Whether this package should be selected by default, considering the category default.
    /// Required packages are always enabled.
    pub fn is_default_enabled(&self, category_default: bool) -> bool {
        if self.required {
            return true;
        }
        self.enabled_by_default.unwrap_or(category_default)
    }
}

/// A command to run during installation
#[derive(Debug, Clone, Deserialize)]
pub struct CommandConfig {
    /// Display name for the command
    pub name: String,
    /// Command and arguments to run
    pub command: Vec<String>,
    /// Whether this command requires sudo (will prompt for password)
    #[serde(default)]
    pub sudo: bool,
}

impl OnboardConfig {
    pub fn load() -> Result<Self, super::error::OnboardError> {
        Self::load_from(DEFAULT_CONFIG_PATH)
    }

    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<Self, super::error::OnboardError> {
        let path = path.as_ref();

        if !path.exists() {
            info!("Config file not found at {:?}, using defaults", path);
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)?;
        let config: OnboardConfig = toml::from_str(&content)?;
        info!("Loaded config from {:?}", path);
        Ok(config)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub title: String,
    pub subtitle: String,
    /// Dry run mode - simulates all operations without making real changes
    /// When true, no system commands are executed, mock data is used, and
    /// reboot transitions to the login screen instead of actually rebooting
    pub dryrun: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            title: "System Setup".to_string(),
            subtitle: "Welcome to your new system".to_string(),
            dryrun: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    pub enabled: bool,
    pub program: String,
    pub args: Vec<String>,
    pub skip_if_connected: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            program: "wifitui".to_string(),
            args: Vec::new(),
            skip_if_connected: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct UserConfig {
    pub groups: Vec<String>,
    pub shell: String,
    pub min_password_length: usize,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            groups: vec!["wheel".to_string()],
            shell: "/bin/bash".to_string(),
            min_password_length: 8,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LocaleConfig {
    pub enabled: bool,
    pub default_locale: String,
    pub available: Vec<String>,
}

impl Default for LocaleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_locale: "en_US.UTF-8".to_string(),
            available: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct KeyboardConfig {
    pub enabled: bool,
    pub default_layout: String,
    pub available: Vec<String>,
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_layout: "us".to_string(),
            available: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PreferencesConfig {
    pub timezone_enabled: bool,
    pub default_timezone: String,
    pub ntp_enabled: bool,
    pub keyring_enabled: bool,
}

impl Default for PreferencesConfig {
    fn default() -> Self {
        Self {
            timezone_enabled: true,
            default_timezone: "UTC".to_string(),
            ntp_enabled: true,
            keyring_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CompletionConfig {
    pub action: String,
    pub remove_initial_session: bool,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            action: "reboot".to_string(),
            remove_initial_session: true,
        }
    }
}
