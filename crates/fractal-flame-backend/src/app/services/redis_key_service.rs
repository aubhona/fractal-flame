#[derive(Clone, Default)]
pub struct RedisKeyService;

impl RedisKeyService {
    pub fn job_status(job_id: &str) -> String {
        format!("job:{}:status", job_id)
    }

    pub fn job_progress(job_id: &str) -> String {
        format!("job:{}:progress", job_id)
    }

    pub fn job_total(job_id: &str) -> String {
        format!("job:{}:total", job_id)
    }

    pub fn job_intermediate_version(job_id: &str) -> String {
        format!("job:{}:intermediate_version", job_id)
    }
}
