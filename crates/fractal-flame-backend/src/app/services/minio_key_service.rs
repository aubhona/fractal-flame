/// Сервис для генерации ключей объектов в MinIO.
#[derive(Clone, Default)]
pub struct MinioKeyService;

impl MinioKeyService {
    /// Ключ для результата рендера: `jobs/{job_id}/result.png`
    pub fn render_result_key(job_id: &str) -> String {
        format!("jobs/{}/result.png", job_id)
    }

    /// Ключ для превью вариации: `previews/{variation_id}_{symmetry}_{gamma}.png`
    pub fn preview_key(variation_id: &str, symmetry: usize, gamma: f64) -> String {
        format!("previews/{}_{}_{:.2}.png", variation_id, symmetry, gamma)
    }
}
