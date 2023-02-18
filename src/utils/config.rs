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
    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

impl From<(i64, i64, i64)> for XpSettings {
    fn from(value: (i64, i64, i64)) -> Self {
        Self {
            delay_anti_spam: value.0,
            min_xp_gain: value.1,
            max_xp_gain: value.2,
        }
    }
}

////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub enum GuildCfgParam {
    SpamDelay,
    MinXpGain,
    MaxXpGain,
}
use GuildCfgParam::*;

impl TryFrom<&str> for GuildCfgParam {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "spam_delay" => Ok(SpamDelay),
            "min_xp_gain" => Ok(MinXpGain),
            "max_xp_gain" => Ok(MaxXpGain),
            _ => Err("Parameters::try_from() returned with error: invalid value"),
        }
    }
}

impl Display for GuildCfgParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpamDelay => write!(f, "spam delay"),
            MinXpGain => write!(f, "min xp gain"),
            MaxXpGain => write!(f, "max xp gain"),
        }
    }
}