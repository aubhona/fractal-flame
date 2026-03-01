use serde::Deserialize;
use std::path::Path;

fn default_samples() -> usize {
    100_000
}
fn default_iter_per_sample() -> usize {
    100
}
fn default_transformation_min_weight() -> f64 {
    0.1
}
fn default_transformation_max_weight() -> f64 {
    1.0
}
fn default_max_threads() -> usize {
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(8)
}
fn default_job_ttl_secs() -> u64 {
    3600
}
fn default_progress_sync_interval_ms() -> u64 {
    100
}
fn default_intermediate_image_interval_ms() -> u64 {
    100
}
fn default_sse_poll_interval_ms() -> u64 {
    100
}
fn default_preview_size() -> usize {
    128
}
fn default_preview_samples() -> usize {
    80_000
}
fn default_preview_iter() -> usize {
    150
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    #[serde(default = "default_samples")]
    pub samples: usize,
    #[serde(default = "default_iter_per_sample")]
    pub iter_per_sample: usize,
    #[serde(default = "default_transformation_min_weight")]
    pub transformation_min_weight: f64,
    #[serde(default = "default_transformation_max_weight")]
    pub transformation_max_weight: f64,
    #[serde(default = "default_max_threads")]
    pub max_threads: usize,
    #[serde(default = "default_job_ttl_secs")]
    pub job_ttl_secs: u64,
    #[serde(default = "default_progress_sync_interval_ms")]
    pub progress_sync_interval_ms: u64,
    #[serde(default = "default_intermediate_image_interval_ms")]
    pub intermediate_image_interval_ms: u64,
    #[serde(default = "default_sse_poll_interval_ms")]
    pub sse_poll_interval_ms: u64,
    #[serde(default = "default_preview_size")]
    pub preview_size: usize,
    #[serde(default = "default_preview_samples")]
    pub preview_samples: usize,
    #[serde(default = "default_preview_iter")]
    pub preview_iter: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            samples: default_samples(),
            iter_per_sample: default_iter_per_sample(),
            transformation_min_weight: default_transformation_min_weight(),
            transformation_max_weight: default_transformation_max_weight(),
            max_threads: default_max_threads(),
            job_ttl_secs: default_job_ttl_secs(),
            progress_sync_interval_ms: default_progress_sync_interval_ms(),
            intermediate_image_interval_ms: default_intermediate_image_interval_ms(),
            sse_poll_interval_ms: default_sse_poll_interval_ms(),
            preview_size: default_preview_size(),
            preview_samples: default_preview_samples(),
            preview_iter: default_preview_iter(),
        }
    }
}

impl Config {
    pub fn from_file(path: Option<impl AsRef<Path>>) -> Result<Self, ConfigError> {
        let path = path
            .map(|p| p.as_ref().to_path_buf())
            .or_else(|| std::env::var("CONFIG_PATH").ok().map(Into::into))
            .unwrap_or_else(|| "config.json".into());

        let contents = std::fs::read_to_string(&path).map_err(|e| ConfigError::ReadFailed {
            path: path.display().to_string(),
            source: e,
        })?;

        let mut config: Self =
            serde_json::from_str(&contents).map_err(|e| ConfigError::ParseFailed {
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
