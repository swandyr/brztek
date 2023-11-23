use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: String,
    pub brzthook: HookCfg,
}

#[derive(Debug, Deserialize)]
pub struct HookCfg {
    pub port: u32,
    pub ip_addr: String,
    pub callback: String,
    pub new_only: bool,
}
