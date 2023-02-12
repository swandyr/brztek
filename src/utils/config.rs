use std::{fmt::Display, io::Write, path::Path, sync::Arc};

use serde::{Deserialize, Serialize};
use serenity::prelude::{RwLock, TypeMapKey};
use tracing::info;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub xp_settings: XpSettings,
}

impl TypeMapKey for Config {
    // Wrapped in a RwLock to allow mutability
    type Value = Arc<RwLock<Self>>;
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

////////////////////////////////////////////////////

#[derive(Debug)]
pub enum GuildCfgParam {
    SpamDelay(i32),
    MinXpGain(i32),
    MaxXpGain(i32),
}

impl TryFrom<String> for GuildCfgParam {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "spam_delay" => Ok(Self::SpamDelay(0)),
            "min_xp_gain" => Ok(Self::MinXpGain(0)),
            "max_xp_gain" => Ok(Self::MaxXpGain(0)),
            _ => Err("Invalid conversion to GuidCfgParam"),
        }
    }
}
