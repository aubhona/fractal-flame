use crate::app::use_cases::{
    get_all_variations_command_handler::GetAllVariationsCommandHandler,
    get_intermediate_result_command_handler::GetIntermediateResultCommandHandler,
    get_render_result_command_handler::GetRenderResultCommandHandler,
    get_variation_preview_command_handler::GetVariationPreviewCommandHandler,
    render_progress_command_handler::RenderProgressCommandHandler,
    run_render_job_command_handler::RunRenderJobCommandHandler,
};
use crate::infra::Dependencies;

pub fn get_run_render_job_command_handler(
    deps: &Dependencies,
) -> Option<RunRenderJobCommandHandler> {
    let minio = deps.minio.as_ref()?;
    Some(RunRenderJobCommandHandler::new(
        deps.config.clone(),
        deps.redis.clone(),
        minio.clone(),
    ))
}

pub fn get_get_render_result_command_handler(
    deps: &Dependencies,
) -> Option<GetRenderResultCommandHandler> {
    let minio = deps.minio.as_ref()?;
    Some(GetRenderResultCommandHandler::new(minio.clone()))
}

pub fn get_get_intermediate_result_command_handler(
    deps: &Dependencies,
) -> Option<GetIntermediateResultCommandHandler> {
    let minio = deps.minio.as_ref()?;
    Some(GetIntermediateResultCommandHandler::new(minio.clone()))
}

pub fn get_render_progress_command_handler(
    deps: &Dependencies,
) -> Option<RenderProgressCommandHandler> {
    let redis = deps.redis.as_ref()?;
    Some(RenderProgressCommandHandler::new(redis.clone()))
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
        deps.config.clone(),
    ))
}
