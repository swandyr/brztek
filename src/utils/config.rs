use std::{io::Write, path::Path, sync::Arc};

use serde::{Deserialize, Serialize};
use serenity::prelude::TypeMapKey;
use tracing::info;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub xp_settings: XpSettings,
}

impl TypeMapKey for Config {
    type Value = Arc<Self>;
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Path::new("config.json");

        let config = if path.is_file() {
            let file_content = std::fs::read_to_string(path)?;
            let config: Config = serde_json::from_str(&file_content)?;
            info!("config.json successfully loaded.");
            config
        } else {
            info!("No config.json found. Creating default configuration.");
            let config = Self::default();
            config.save()?;
            config
        };

        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = std::fs::File::create("config.json")?;
        let serialized = serde_json::to_string_pretty(&self)?;
        write!(&mut file, "{serialized}")?;
        info!("config.json successfully written.");

        Ok(())
    }
}

// Xp parameters

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct XpSettings {
    pub min_xp_gain: i64,
    pub max_xp_gain: i64,
    pub delay_anti_spam: i64,
}

impl Default for XpSettings {
    fn default() -> Self {
        Self {
            min_xp_gain: 15,
            max_xp_gain: 25,
            delay_anti_spam: 30,
        }
    }
}
