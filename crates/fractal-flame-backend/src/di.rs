use crate::app::use_cases::{
    get_all_variations_command_handler::GetAllVariationsCommandHandler,
    get_variation_preview_command_handler::GetVariationPreviewCommandHandler,
};
use crate::infra::Dependencies;

pub fn get_get_all_variations_command_handler(
    deps: &Dependencies,
) -> GetAllVariationsCommandHandler {
    GetAllVariationsCommandHandler::new(deps.transformations.clone())
}

pub fn get_get_variation_preview_command_handler(
    deps: &Dependencies,
) -> GetVariationPreviewCommandHandler {
    GetVariationPreviewCommandHandler::new(
        deps.preview_cache.clone(),
        deps.config.max_threads,
    )
}
