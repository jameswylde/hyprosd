use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub osd: OsdConfig,
    pub theme: ThemeConfig,
    #[serde(default)]
    pub backend: BackendConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsdConfig {
    pub width: i32,
    pub height: i32,
    #[serde(default = "default_lock_size")]
    pub lock_size: i32,
    #[serde(default = "default_bar_height")]
    pub bar_height: i32,
    pub timeout_ms: u64,
    pub offset_y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub background: String,
    pub foreground: String,
    pub accent: String,
    pub font_family: String,
    pub font_size: i32,
    pub corner_radius: i32,
    pub padding: i32,
    pub icon_size: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackendConfig {
    #[serde(default = "default_brightness_path")]
    pub brightness_path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            osd: OsdConfig {
                width: 288,
                height: 53,
                lock_size: 58,
                bar_height: 1,
                timeout_ms: 1600,
                offset_y: 60,
            },
            theme: ThemeConfig {
                background: "#181818cc".to_string(),
                foreground: "#ffffff".to_string(),
                accent: "#88c0ff".to_string(),
                font_family: "Google Sans, sans-serif".to_string(),
                font_size: 17,
                corner_radius: 20,
                padding: 18,
                icon_size: 26,
            },
            backend: BackendConfig {
                brightness_path: None,
            },
        }
    }
}

impl Config {
    pub fn load_or_init() -> anyhow::Result<Self> {
        // config structs are ready for a file later, but defaults keep the first release simple
        Ok(Self::default())
    }
}

fn default_brightness_path() -> Option<PathBuf> {
    None
}

fn default_lock_size() -> i32 {
    56
}

fn default_bar_height() -> i32 {
    6
}
