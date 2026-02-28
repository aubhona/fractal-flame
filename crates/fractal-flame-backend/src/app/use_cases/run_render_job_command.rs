pub struct RunRenderJobCommand {
    pub job_id: String,
    pub variation_ids: Vec<String>,
    pub symmetry: usize,
    pub gamma: f64,
    pub width: usize,
    pub height: usize,
}
