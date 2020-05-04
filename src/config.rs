use anyhow::Result as AnyResult;
use tokio::fs;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    pub token: String,
}

impl Config {
    pub async fn load<P: AsRef<Path>>(path: P) -> AnyResult<Self> {
        let cfg_str = fs::read_to_string(path).await?;
        Ok(serde_json::from_str(&cfg_str)?)
    }

    pub async fn load_or_create<P: AsRef<Path>>(path: P) -> AnyResult<Self> {
        let cfg_str = match fs::read_to_string(&path).await {
            Ok(cfg) => cfg,
            Err(e) => {
                if e.kind() == tokio::io::ErrorKind::NotFound {
                    let empty_cfg = Config::default();
                    empty_cfg.save(&path).await?;
                    return Ok(empty_cfg);
                }
                return Err(e.into());
            }
        };
        Ok(serde_json::from_str(&cfg_str)?)
    }

    pub async fn save<P: AsRef<Path>>(&self, path: P) -> AnyResult<()> {
        let cfg_str = serde_json::to_string_pretty(self)?;
        Ok(fs::write(path, cfg_str.as_bytes()).await?)
    }
}

pub fn default_config_path() -> PathBuf {
    dirs::config_dir().map(|mut path| {
        path.push("github-labelexim.json");
        path
    }).expect("Cannot find config path")
}
