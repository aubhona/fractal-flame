use crate::app::use_cases::{
    get_all_variations_command_handler::GetAllVariationsCommandHandler,
    get_variation_preview_command_handler::GetVariationPreviewCommandHandler,
    run_render_job_command_handler::RunRenderJobCommandHandler,
};
use crate::infra::Dependencies;

pub fn get_run_render_job_command_handler(
    deps: &Dependencies,
) -> Option<RunRenderJobCommandHandler> {
    let minio = deps.minio.as_ref()?;
    Some(RunRenderJobCommandHandler::new(
        deps.transformations.clone(),
        deps.config.clone(),
        deps.redis.clone(),
        minio.clone(),
    ))
}

pub fn get_get_all_variations_command_handler(
    deps: &Dependencies,
) -> GetAllVariationsCommandHandler {
    GetAllVariationsCommandHandler::new(deps.transformations.clone())
}

pub fn get_get_variation_preview_command_handler(
    deps: &Dependencies,
) -> Option<GetVariationPreviewCommandHandler> {
    let minio = deps.minio.as_ref()?;
    Some(GetVariationPreviewCommandHandler::new(
        minio.clone(),
        deps.config.max_threads,
    ))
}
