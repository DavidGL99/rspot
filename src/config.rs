use std::fs;
use std::path::PathBuf;

#[derive(serde::Deserialize, Default)]
pub struct Config {
    pub window: WindowConfig,
    pub colors: ColorsConfig,
    pub font: FontConfig,
}

#[derive(serde::Deserialize)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
}

#[derive(serde::Deserialize)]
pub struct ColorsConfig {
    pub background: String,
    pub opacity: f32,
    pub selected_item_color: String,
}

#[derive(serde::Deserialize)]
pub struct FontConfig {
    pub font_size: u32,
    pub font_color: String,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 500,
            height: 350,
        }
    }
}

impl Default for ColorsConfig {
    fn default() -> Self {
        Self {
            background: "#2b2b2b".to_string(),
            opacity: 0.9,
            selected_item_color: "#5294e2".to_string(),
        }
    }
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            font_color: "#ffffff".to_string(),
            font_size: 14,
        }
    }
}

pub fn load_config() -> Config {
    // 1. construir la ruta ~/.config/rspot/config.toml
    let home = std::env::var("HOME").unwrap();
    let path = PathBuf::from(home).join(".config/rspot/config.toml");
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap();
        println!("using file config");
        return toml::from_str(&content).unwrap_or_default();
    }
    println!("using default config");
    Config::default()
}
