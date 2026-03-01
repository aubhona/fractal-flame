/// Service for generating object keys in MinIO.
#[derive(Clone, Default)]
pub struct MinioKeyService;

impl MinioKeyService {
    /// Key for render result: `jobs/{job_id}/result.png`
    pub fn render_result_key(job_id: &str) -> String {
        format!("jobs/{}/result.png", job_id)
    }

    /// Key for intermediate snapshot: `jobs/{job_id}/intermediate.png`
    pub fn intermediate_key(job_id: &str) -> String {
        format!("jobs/{}/intermediate.png", job_id)
    }

    /// Key for variation preview: `previews/{variation_id}_{symmetry}_{gamma}.png`
    pub fn preview_key(variation_id: &str, symmetry: usize, gamma: f64) -> String {
        format!("previews/{}_{}_{:.2}.png", variation_id, symmetry, gamma)
    }
}
