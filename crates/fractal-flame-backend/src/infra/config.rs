use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub samples: usize,
    pub iter_per_sample: usize,
    pub transformation_min_weight: f64,
    pub transformation_max_weight: f64,
    #[serde(default)]
    pub max_threads: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            samples: 100_000,
            iter_per_sample: 100,
            transformation_min_weight: 0.1,
            transformation_max_weight: 1.0,
            max_threads: std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(8),
        }
    }
}

impl Config {
    pub fn from_file(path: Option<impl AsRef<Path>>) -> Result<Self, ConfigError> {
        let path = path
            .map(|p| p.as_ref().to_path_buf())
            .or_else(|| std::env::var("CONFIG_PATH").ok().map(Into::into))
            .unwrap_or_else(|| "config.json".into());

        let contents = std::fs::read_to_string(&path)
            .map_err(|e| ConfigError::ReadFailed {
                path: path.display().to_string(),
                source: e,
            })?;

        let mut config: Self = serde_json::from_str(&contents).map_err(|e| ConfigError::ParseFailed {
            path: path.display().to_string(),
            source: e,
        })?;

        if config.max_threads == 0 {
            config.max_threads = std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(8);
        }

        Ok(config)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config from {path}: {source}")]
    ReadFailed {
        path: String,
        source: std::io::Error,
    },
    #[error("Failed to parse config from {path}: {source}")]
    ParseFailed {
        path: String,
        source: serde_json::Error,
    },
}
